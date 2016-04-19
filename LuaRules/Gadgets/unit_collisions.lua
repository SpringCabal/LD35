function gadget:GetInfo()
	return {
		name = "Unit collisions",
		desc = "Handle setting collision/blocking of units",
		author = "gajop",
		date = "April 2016",
		license = "GNU GPL, v2 or later",
		layer = 1,
		enabled = true
	}
end

if (gadgetHandler:IsSyncedCode()) then

function gadget:UnitCreated(unitID, unitDefID)
	local unitDef = UnitDefs[unitDefID]
	if unitDef.name == "gatesmoth" then
-- 		Spring.SetUnitBlocking(unitID, false, false, false, false, false, false, false)
	elseif unitDef.customParams.wall and unitDef.customParams.wall == true then
		Spring.SetUnitCollisionVolumeData(unitID, 
			0, 0, 0, 
			0, 0, 0, 
			0, 0, 0);
		Spring.SetUnitBlocking(unitID, false, false, false, false, false, false, false)
	end
end

end
