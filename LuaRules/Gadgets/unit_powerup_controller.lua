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

local PICKUP_RANGE = 50

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
		wispEnv = Spring.UnitScript.GetScriptEnv(wispID)
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetAllPiecesInvisibleNoThread)
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
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

function gadget:GameFrame()
	if not wispID then
		return
	end
	
	if oldSpiritMode == nil then
		oldSpiritMode = Spring.GetGameRulesParam("spiritMode")
	end
	if oldSpiritMode == 0 and Spring.GetGameRulesParam("spiritMode") == 1 then
		oldSpiritMode = 1
		Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetAllPiecesInvisibleNoThread)
	elseif oldSpiritMode == 1 and Spring.GetGameRulesParam("spiritMode") == 0 then
		oldSpiritMode = 0
		for ruleName, _ in pairs(Spring.GetGameRulesParams()) do
			if ruleName:find("has_") then
				local bodyPart = ruleName:sub(#"has_" + 1)
				Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, bodyPart, true)
			end
		end
	end

	local x, _, z = Spring.GetUnitPosition(wispID)
	for _, unitID in pairs(Spring.GetUnitsInCylinder(x, z, 50)) do
		local unitDef = UnitDefs[Spring.GetUnitDefID(unitID)]
		if unitDef.customParams.bodypart ~= nil then
			Spring.Log("powerup", LOG.NOTICE, "Picked up body part: " .. unitDef.customParams.bodypart)
			Spring.SetGameRulesParam("has_" .. unitDef.customParams.bodypart, 1)
			Spring.DestroyUnit(unitID)

			if Spring.GetGameRulesParam("spiritMode") == 1 then
				Spring.UnitScript.CallAsUnit(wispID, wispEnv.SetPieceVisibleNoThread, unitDef.customParams.bodypart, true)
			end
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

