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
local PER_FRAME_CHANGE = 1

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil
local wispMoving

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

local function GiveClampedMoveGoal(unitID, x, z, radius)
	radius = radius or 16
	local cx, cz = Spring.Utilities.ClampPosition(x, z)
	local cy = Spring.GetGroundHeight(cx, cz)
	--Spring.MarkerAddPoint(cx, cy, cz)
	Spring.SetUnitMoveGoal(unitID, cx, cy, cz, radius, nil, true) -- The last argument is whether the goal is raw
	return true
end

local function MoveUnit(unitID, x, z, range, radius)
	if not (unitID and Spring.ValidUnitID(unitID)) then
		return
	end
	local speed = Spring.GetUnitRulesParam(unitID, "selfMoveSpeedChange") or 1
	range = (range or 1000)*speed

	local ux, uy, uz = Spring.GetUnitPosition(unitID)

	local moveVec = Vector.Norm(range, {x, z})

	GiveClampedMoveGoal(unitID, moveVec[1] + ux, moveVec[2] + uz, radius)
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

function _ChangeHeightmap(startX, startZ, delta, radius)
	local kernel = _PrecalculateKernel(radius)
	radius = radius - radius % Game.squareSize
	startX = startX - startX % Game.squareSize
	startZ = startZ - startZ % Game.squareSize
	for x = 0, 2*radius, Game.squareSize do
		for z = 0, 2*radius, Game.squareSize do
			local gh = Spring.GetGroundHeight(x + startX, z + startZ)
			local ogh = Spring.GetGroundOrigHeight(x + startX, z + startZ)
			local maxGH, minGH = ogh + MAX_HEIGHT_CHANGE, ogh - MAX_HEIGHT_CHANGE
			local d = delta * kernel[1 + x + z * 2*radius]
			if d > 0 then
				d = math.min(d, maxGH - gh)
			else
				d = math.max(d, minGH - gh)
			end
			
			Spring.AddHeightMap(x + startX, z + startZ,  d)
		end
	end
end

function ChangeHeightmap(x, z, delta, radius)
	if Spring.GetGameFrame() - heightmapChangedFrame > PER_FRAME_CHANGE then
		heightmapChangedFrame = Spring.GetGameFrame()
		Spring.SetHeightMapFunc(_ChangeHeightmap, x - radius, z - radius, delta, radius)
	end
end

-------------------------------------------------------------------
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
	if not wispID then
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

		ChangeHeightmap(x, z, 5, 64)
	elseif msg_table[1] == 'dec_heightmap' then -- RMB
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])

		ChangeHeightmap(x, z, -5, 64)
	elseif msg_table[1] == 'switch_form' then
		Spring.SetGameRulesParam("spiritMode", 1 - Spring.GetGameRulesParam("spiritMode"))
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

function gadget:GameFrame(frame)
	if wispID then
		local x, y, z = Spring.GetUnitPosition(wispID)

		if (movementMessage and movementMessage.frame + 2 > frame) then
			MoveUnit(wispID, movementMessage.x, movementMessage.z)
			wispMoving = true
		else

			local vx, _, vz = Spring.GetUnitVelocity(wispID)
			if vx then
				local speed = Vector.AbsVal(vx, vz)
				if wispMoving or (speed > 6) then
					MoveUnit(wispID, vx, vz, 20)
					wispMoving = false
				end
			end

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

