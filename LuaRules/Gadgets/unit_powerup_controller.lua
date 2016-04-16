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


-- local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local PICKUP_RANGE = 50

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

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
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

function gadget:GameFrame()
	if wispID == nil then
		return
	end

	local x, _, z = Spring.GetUnitPosition(wispID)
	for _, unitID in pairs(Spring.GetUnitsInCylinder(x, z, 50)) do
		local unitDef = UnitDefs[Spring.GetUnitDefID(unitID)]
		if unitDef.customParams.bodyPart ~= nil then
			Spring.Log("powerup", LOG.NOTICE, "Picked up body part: " .. unitDef.customParams.bodyPart)
			Spring.SetGameRulesParam("has_" .. unitDef.customParams.bodyPart, 1)
			Spring.DestroyUnit(unitID)
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

