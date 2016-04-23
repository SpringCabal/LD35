--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Game End",
		desc      = "Does gameover stuff",
		author    = "gajop",
		date      = "April 2016",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true  --  loaded by default?
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local npcWispDefID = UnitDefNames["npcwisp"].id

local spawnWisps = false
local spawnedAmount = 0
local spawnedTime = 0
local SPAWN_FRAME_CD = 10
local SPAWN_AMOUNT = 500
local SPAWN_DISTANCE, SPAWN_DISTANCE_MIN = 300, 100

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
	
	Spring.SetGameRulesParam("game_over", 0)
end

local bodyParts = { "eyes", "ears", "arms", "legs", "nose", "mouth" }

function gadget:GameFrame()
	if spawnWisps then
		local frame = Spring.GetGameFrame()
		if spawnedAmount < SPAWN_AMOUNT then
			if frame - spawnedTime > SPAWN_FRAME_CD then
				local x, y, z = Spring.GetUnitPosition(wispID)
				local dx, dz = (0.5 - math.random()) * SPAWN_DISTANCE * 2, (0.5 - math.random()) * SPAWN_DISTANCE * 2
				if dx > 0 then
					dx = dx + SPAWN_DISTANCE_MIN
				else
					dx = dx - SPAWN_DISTANCE_MIN
				end
				if dz > 0 then
					dz = dz + SPAWN_DISTANCE_MIN
				else
					dz = dz - SPAWN_DISTANCE_MIN
				end
				
				local unitID = Spring.CreateUnit(npcWispDefID, x + dx, y, z + dz, 0, 0)
				Spring.GiveOrderToUnit(unitID, CMD.MOVE, { x, y, z }, {})
				spawnedAmount = spawnedAmount + 1
				GG.SaveWisp(unitID)
			end
		else
			Spring.SetGameRulesParam("game_over_sequence", 1)
		end
	end
	
	local hasAllParts = true
	for _, bodyPart in pairs(bodyParts) do
		if Spring.GetGameRulesParam("has_" .. bodyPart) ~= 1 then
			hasAllParts = false
			break
		end
	end
-- 	hasAllParts = hasAllParts or Spring.GetGameRulesParam("has_eyes") == 1
	if hasAllParts and Spring.GetGameRulesParam("game_over") ~= 1 then
		Spring.SetGameRulesParam("game_over", 1)
		local alive_wisps = Spring.GetGameRulesParam("alive_wisps") or 0
		local saved_wisps = Spring.GetGameRulesParam("saved_wisps") or 0
		local killed_wisps = Spring.GetGameRulesParam("killed_wisps") or 0
		if alive_wisps ~= 0 and saved_wisps == 0 and killed_wisps == 0 then
			Spring.SetGameRulesParam("game_over_type", 0) -- Welcome back, Master.
			Spring.SetGameRulesParam("game_over_sequence", 1)
		elseif alive_wisps == 0 and saved_wisps ~= 0 and killed_wisps == 0 then
			Spring.SetGameRulesParam("game_over_sequence", 0)
			Spring.SetGameRulesParam("spiritMode", 1)
			Spring.SetGameRulesParam("game_over_type", 1) -- The souls have been freed.
			spawnWisps = true
			Spring.MoveCtrl.Enable(wispID)
		elseif alive_wisps == 0 and saved_wisps == 0 and killed_wisps ~= 0 then
			Spring.SetGameRulesParam("game_over_type", 2) -- Their sacrifice is accepted.
			Spring.SetGameRulesParam("game_over_sequence", 1)
		else
			Spring.SetGameRulesParam("game_over_type", 3) -- You have achieved nothing noteworthy.
			Spring.SetGameRulesParam("game_over_sequence", 1)
		end
	end
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
