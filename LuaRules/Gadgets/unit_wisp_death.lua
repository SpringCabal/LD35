--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Wisp death",
		desc	= "Kills and cleanups wisps",
		author	= "gajop",
		date	= "17 April 2016",
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

local npcWispDefID = UnitDefNames["npcwisp"].id
local DEFAULT_DESTROY_TIME = 30 * 1.5

-------------------------------------------------------------------
-------------------------------------------------------------------

function KillWisp(unitID, destroyTime)
	destroyTime = destroyTime or DEFAULT_DESTROY_TIME
	Spring.SetUnitRulesParam(unitID, "is_dying", 1)
	Spring.SetUnitRulesParam(unitID, "killed_frame", Spring.GetGameFrame())
	Spring.SetUnitRulesParam(unitID, "destroy_frame", Spring.GetGameFrame() + destroyTime)
end

function gadget:GameFrame()
	local frame = Spring.GetGameFrame()
	for _, unitID in pairs(Spring.GetAllUnits()) do
		if Spring.GetUnitRulesParam(unitID, "is_dying") == 1 then
			local destroy_frame = Spring.GetUnitRulesParam(unitID, "destroy_frame")
			if destroy_frame == frame then
				Spring.DestroyUnit(unitID)
			end
		end
	end
end

GG.KillWisp = KillWisp

end
