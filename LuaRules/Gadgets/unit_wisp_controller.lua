--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Wisp",
		desc	= "Wisp control gadget.",
		author	= "gajop",
		date	= "16 April 2016",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end


-------------------------------------------------------------------
-- SYNCED
-------------------------------------------------------------------
if gadgetHandler:IsSyncedCode() then

-------------------------------------------------------------------
-------------------------------------------------------------------

local sqrt = math.sqrt

local Vector = Spring.Utilities.Vector

-------------------------------------------------------------------
-------------------------------------------------------------------


local MAX_HEIGHT_CHANGE = 200
local PER_FRAME_CHANGE = 0
local HEIGHT_CHANGE_PER_FRAME = 7

-- How far ahead of the wisp the move goal is placed.  ClampPosition() clamps x
-- and z independently, so a long lookahead near a map edge collapsed diagonal
-- input onto a single axis; keep it short, it is re-issued every frame anyway.
local MOVE_LOOKAHEAD = 120
-- Frames a movement message stays valid.  The widget sends one per frame, but
-- they arrive through the network layer, so a 2 frame window dropped inputs
-- whenever delivery hiccuped and the wisp stuttered to a halt.
local MOVE_STALE_FRAMES = 6

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil
local wispMoving

local FORM_CHANGE_COOLDOWN = 30*2

local movementMessage

local heightmapChangedFrame = 0
-------------------------------------------------------------------
-------------------------------------------------------------------

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

-- Spring.Utilities.ClampPosition() clamps x and z independently, which rotates
-- an out-of-bounds goal onto an axis and makes diagonal input near a map edge
-- read as cardinal.  Shorten the offset instead so the heading is preserved.
local function ClampOffsetToMap(ux, uz, dx, dz)
	local scale = 1
	local function limit(origin, delta, lo, hi)
		if delta > 0 and origin + delta > hi then
			scale = math.min(scale, (hi - origin) / delta)
		elseif delta < 0 and origin + delta < lo then
			scale = math.min(scale, (lo - origin) / delta)
		end
	end
	limit(ux, dx, 1, Game.mapSizeX - 1)
	limit(uz, dz, 1, Game.mapSizeZ - 1)
	scale = math.max(scale, 0)
	return ux + dx * scale, uz + dz * scale
end

local function GiveClampedMoveGoal(unitID, x, z, radius)
	radius = radius or 16
	local cx, cz = Spring.Utilities.ClampPosition(x, z)
	local cy = Spring.GetGroundHeight(cx, cz)
	--Spring.MarkerAddPoint(cx, cy, cz)
	local _, height, _ = Spring.GetUnitPosition(unitID)
	-- Raw goal (last argument): head straight where the player pointed instead of
	-- letting the pathfinder plan a detour around whatever is in the way.
	Spring.SetUnitMoveGoal(unitID, cx, cy, cz, radius, nil, true)
	return true
end

local function MoveUnit(unitID, x, z, range, radius)
	if not (unitID and Spring.ValidUnitID(unitID)) then
		return
	end
	local speed = Spring.GetUnitRulesParam(unitID, "selfMoveSpeedChange") or 1
	range = (range or MOVE_LOOKAHEAD)*speed

	local ux, uy, uz = Spring.GetUnitPosition(unitID)

	local moveVec = Vector.Norm(range, {x, z})
	local gx, gz = ClampOffsetToMap(ux, uz, moveVec[1], moveVec[2])

	GiveClampedMoveGoal(unitID, gx, gz, radius)
end


function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
	end
end

function gadget:UnitDestroyed(unitID)
	if wispID == unitID then
		wispID = nil
	end
end

function gadget:GameStart()
	gameStarted = true
end

function gadget:Initialize()
	if Spring.GetGameRulesParam("spiritMode") == nil then
		if Spring.GetGameRulesParam("gameMode") ~= "develop" then
			Spring.SetGameRulesParam("spiritMode", 1)
		else
			Spring.SetGameRulesParam("spiritMode", 0)
		end
	end
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

local kernels = {}
function _PrecalculateKernel(radius)
	if kernels[radius] then return kernels[radius] end
	local kernel = {}
	kernels[radius] = kernel
	for x = 0, 2*radius do
		local dx = radius - x
		for y = 0, 2*radius do
			local dy = radius - y
			kernel[1 + x + y * 2*radius] = 2500 / (dx * dx + dy * dy + 2500)
		end
	end
	return kernel
end

function _ChangeHeightmap(startX, startZ, delta, radius, height)
	local kernel = _PrecalculateKernel(radius)
	radius = radius - radius % Game.squareSize
	startX = startX - startX % Game.squareSize
	startZ = startZ - startZ % Game.squareSize
	for x = 0, 2*radius, Game.squareSize do
		for z = 0, 2*radius, Game.squareSize do
			local d = delta * kernel[1 + x + z * 2*radius]
			local gh = Spring.GetGroundHeight(x + startX, z + startZ)
-- 			local ogh = Spring.GetGroundOrigHeight(x + startX, z + startZ)
-- 			local maxGH, minGH = ogh + MAX_HEIGHT_CHANGE, ogh - MAX_HEIGHT_CHANGE
-- 			if d > 0 then
-- 				d = math.min(d, maxGH - gh)
-- 			else
-- 				d = math.max(d, minGH - gh)
-- 			end
			if delta < 0 then
				return
-- 				height = Spring.GetGroundOrigHeight(x + startX, z + startZ)
			end
		
			if height > gh then
				d = math.min(d, height - gh)
			else
				d = -math.min(d, gh - height)
			end
			Spring.AddHeightMap(x + startX, z + startZ,  d)
		end
	end
end

function ChangeHeightmap(x, z, delta, radius, height)
	if Spring.GetGameFrame() - heightmapChangedFrame > PER_FRAME_CHANGE then
		heightmapChangedFrame = Spring.GetGameFrame()
		Spring.SetHeightMapFunc(_ChangeHeightmap, x - radius, z - radius, delta, radius, height)
	end
end

-------------------------------------------------------------------
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
	if not wispID or Spring.GetGameRulesParam("game_over") == 1 then
		return
	end
	local msg_table = explode('|', msg)

	if msg_table[1] == 'movement' then
		local x = tonumber(msg_table[2])
		local z = tonumber(msg_table[3])

		if x == 0 and z == 0 then
			movementMessage = false
		else
			movementMessage = {
				frame = Spring.GetGameFrame(),
				x = x,
				z = z
			}
		end
	end

	if msg_table[1] == 'inc_heightmap' then --LMB
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])
		local height = tonumber(msg_table[5])

-- 		ChangeHeightmap(x, z, HEIGHT_CHANGE_PER_FRAME, 64, height)
	elseif msg_table[1] == 'dec_heightmap' then -- RMB
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])
		local height = tonumber(msg_table[5])

-- 		ChangeHeightmap(x, z, -HEIGHT_CHANGE_PER_FRAME, 64, height)
	elseif msg_table[1] == 'switch_form' then
		local formChangeFrame = Spring.GetGameRulesParam("formChangeFrame") or 0
		local frame = Spring.GetGameFrame()
		if frame - formChangeFrame > FORM_CHANGE_COOLDOWN and Spring.GetGameRulesParam("has_eyes") == 1 then
			Spring.SetGameRulesParam("formChangeFrame", frame)

			Spring.SetGameRulesParam("spiritMode", 1 - Spring.GetGameRulesParam("spiritMode"))
			local x, y, z = Spring.GetUnitPosition(wispID)
			Spring.PlaySoundFile("sounds/changestate.ogg", 1, 'sfx')
		end
	elseif msg_table[1] == 'pull_lever' then
		local unitID = tonumber(msg_table[2])
		local active = Spring.GetUnitStates(unitID).active
		if active then
			Spring.GiveOrderToUnit(unitID, CMD.ONOFF, { 0 }, {})
		else
			Spring.GiveOrderToUnit(unitID, CMD.ONOFF, { 1 }, {})
		end
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

function gadget:GameFrame(frame)
	if wispID then
		-- force spiritMode on start if no eyes 
		if Spring.GetGameRulesParam("spiritMode") ~= 1 and
			Spring.GetGameRulesParam("has_eyes") ~= 1 then
			Spring.SetGameRulesParam("spiritMode", 1)
		end
		
		
		local x, y, z = Spring.GetUnitPosition(wispID)

		local vx, vy, vz = Spring.GetUnitVelocity(wispID)
		if vy > 0 then
			vy = 0
		end
		Spring.SetUnitVelocity(wispID, vx, vy, vz)
		
		if (movementMessage and movementMessage.frame + MOVE_STALE_FRAMES > frame) then
			MoveUnit(wispID, movementMessage.x, movementMessage.z)
			wispMoving = true
		elseif wispMoving then
			-- Only brake on the transition out of movement.  Issuing STOP on every
			-- idle frame kept cancelling the order the wisp had just been given.
			Spring.GiveOrderToUnit(wispID, CMD.STOP,{},{})
			local vx, _, vz = Spring.GetUnitVelocity(wispID)
			if vx and Vector.AbsVal(vx, vz) > 6 then
				MoveUnit(wispID, vx, vz, 20)
			end

			wispMoving = false
			movementMessage = false
		end
		



		Spring.SetGameRulesParam("wisp_x", x)
		Spring.SetGameRulesParam("wisp_y", y)
		Spring.SetGameRulesParam("wisp_z", z)
	end
end


-------------------------------------------------------------------
-- UNSYNCED
-------------------------------------------------------------------
else
-------------------------------------------------------------------

  return

-------------------------------------------------------------------
end

