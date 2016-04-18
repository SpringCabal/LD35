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
	Spring.SetUnitRulesParam(unitID, "killed_frame", Spring.GetGameFrame(), {public=true})
	Spring.SetUnitRulesParam(unitID, "destroy_frame", Spring.GetGameFrame() + destroyTime, {public=true})
end

function SaveWisp(unitID, destroyTime)
	destroyTime = destroyTime or DEFAULT_DESTROY_TIME
	Spring.SetUnitRulesParam(unitID, "is_dying", 1)
	Spring.SetUnitRulesParam(unitID, "is_being_saved", 1)
	Spring.SetUnitRulesParam(unitID, "killed_frame", Spring.GetGameFrame(), {public=true})
	Spring.SetUnitRulesParam(unitID, "destroy_frame", Spring.GetGameFrame() + destroyTime, {public=true})
end

function gadget:GameFrame()
	local frame = Spring.GetGameFrame()
	for _, unitID in pairs(Spring.GetAllUnits()) do
		if Spring.GetUnitRulesParam(unitID, "is_dying") == 1 then
			local destroy_frame = Spring.GetUnitRulesParam(unitID, "destroy_frame")
			if destroy_frame == frame then
				local is_being_saved = Spring.GetUnitRulesParam(unitID, "is_being_saved") or 0
				if is_being_saved == 1 then
					local v =Spring.GetGameRulesParam("saved_wisps") or 0
					Spring.SetGameRulesParam("saved_wisps", v + 1)
				else
					local v = Spring.GetGameRulesParam("killed_wisps") or 0
					Spring.SetGameRulesParam("killed_wisps", v + 1)
				end
				Spring.DestroyUnit(unitID)
			end
		end
	end
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if npcWispDefID == unitDefID then
		local v =  Spring.GetGameRulesParam("alive_wisps") or 0
		Spring.SetGameRulesParam("alive_wisps", v + 1)
	end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if npcWispDefID == unitDefID then
		local v = Spring.GetGameRulesParam("alive_wisps") or 0
		Spring.SetGameRulesParam("alive_wisps", v - 1)
	end
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end

	Spring.SetGameRulesParam("alive_wisps", 0)
	Spring.SetGameRulesParam("saved_wisps", 0)
	Spring.SetGameRulesParam("killed_wisps", 0)
end

GG.KillWisp = KillWisp
GG.SaveWisp = SaveWisp

end
