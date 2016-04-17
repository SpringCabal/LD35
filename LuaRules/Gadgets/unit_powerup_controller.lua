--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Powerups",
		desc	= "Powerup control gadget.",
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


local wispDefID = UnitDefNames["wisp"].id
local wispID = nil
local wispEnv

local npcWispDefID = UnitDefNames["npcwisp"].id

local PICKUP_RANGE = 50

local bodyParts = {}

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
		wispEnv = Spring.UnitScript.GetScriptEnv(wispID)
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetAllPiecesInvisibleNoThread)
	end
	if npcWispDefID == unitDefID then
		local env = Spring.UnitScript.GetScriptEnv(unitID)
		Spring.UnitScript.CallAsUnit(unitID, env.SetAllPiecesInvisibleNoThread)
	end
	local unitDef = UnitDefs[unitDefID]
	if unitDef.customParams.bodypart ~= nil then
		bodyParts[unitID] = true
	end
end

function gadget:UnitDestroyed(unitID)
	if wispID == unitID then
		wispID = nil
	end
	bodyParts[unitID] = nil
end

function gadget:GameStart()
	gameStarted = true
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

function gadget:GameFrame()
	if not wispID then
		return
	end
	
	-- hide/show body parts
	if oldSpiritMode ~= 1 and Spring.GetGameRulesParam("spiritMode") == 1 then
		oldSpiritMode = 1
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetAllPiecesInvisibleNoThread)
	elseif oldSpiritMode ~= 0 and Spring.GetGameRulesParam("spiritMode") == 0 then
		oldSpiritMode = 0
		for ruleName, value in pairs(Spring.GetGameRulesParams()) do
			if ruleName:find("has_") and value == 1 then
				local bodyPart = ruleName:sub(#"has_" + 1)
				Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, bodyPart, true)
			end
		end
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, "body", true)
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, "head", true)
	end

	-- pickup body parts
	local x, _, z = Spring.GetUnitPosition(wispID)
	for _, unitID in pairs(Spring.GetUnitsInCylinder(x, z, 50)) do
		if bodyParts[unitID] ~= nil then
			local unitDef = UnitDefs[Spring.GetUnitDefID(unitID)]
			Spring.Log("powerup", LOG.NOTICE, "Picked up body part: " .. unitDef.customParams.bodypart)
			Spring.SetGameRulesParam("has_" .. unitDef.customParams.bodypart, 1)
			Spring.DestroyUnit(unitID)

			if Spring.GetGameRulesParam("spiritMode") == 0 then
				Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, unitDef.customParams.bodypart, true)
			end
		end
	end

	-- rotate body parts
	for _, unitID in pairs(Spring.GetAllUnits()) do
		if bodyParts[unitID] then
			local p, y, r = Spring.GetUnitRotation(unitID)
			Spring.SetUnitRotation(unitID, p, y + 0.1, r)
		end
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

