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
local pending = {}

-------------------------------------------------------------------
-------------------------------------------------------------------

function KillWisp(unitID, destroyTime)
	destroyTime = destroyTime or DEFAULT_DESTROY_TIME
	Spring.SetUnitRulesParam(unitID, "is_dying", 1)
	Spring.SetUnitRulesParam(unitID, "killed_frame", Spring.GetGameFrame(), {public=true})
	Spring.SetUnitRulesParam(unitID, "destroy_frame", Spring.GetGameFrame() + destroyTime, {public=true})
	pending[unitID] = Spring.GetGameFrame() + destroyTime
	local x, y, z = Spring.GetUnitPosition(unitID)
	Spring.PlaySoundFile("sounds/death.ogg", 100, x, y, z)
end

function SaveWisp(unitID, destroyTime)
	destroyTime = destroyTime or DEFAULT_DESTROY_TIME
	Spring.SetUnitRulesParam(unitID, "is_dying", 1)
	Spring.SetUnitRulesParam(unitID, "is_being_saved", 1)
	Spring.SetUnitRulesParam(unitID, "killed_frame", Spring.GetGameFrame(), {public=true})
	Spring.SetUnitRulesParam(unitID, "destroy_frame", Spring.GetGameFrame() + destroyTime, {public=true})
	pending[unitID] = Spring.GetGameFrame() + destroyTime
	local x, y, z = Spring.GetUnitPosition(unitID)
	local volume = 100
	if Spring.GetGameRulesParam("game_over") == 1 then
		volume = 3
	end
	Spring.PlaySoundFile("sounds/freed.ogg", volume, x, y, z)
end

function gadget:GameFrame(frame)
	-- Only scheduled deaths need polling; other units cannot enter this state
	-- except through KillWisp/SaveWisp above.
	for unitID, destroyFrame in pairs(pending) do
		if destroyFrame == frame then
			pending[unitID] = nil
			local saved = Spring.GetUnitRulesParam(unitID, "is_being_saved") == 1
			local key = saved and "saved_wisps" or "killed_wisps"
			Spring.SetGameRulesParam(key, (Spring.GetGameRulesParam(key) or 0) + 1)
			Spring.DestroyUnit(unitID)
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
	pending[unitID] = nil
	if npcWispDefID == unitDefID then
		local v = Spring.GetGameRulesParam("alive_wisps") or 0
		Spring.SetGameRulesParam("alive_wisps", v - 1)
	end
end

function gadget:Initialize()
	Spring.SetGameRulesParam("alive_wisps", 0)
	Spring.SetGameRulesParam("saved_wisps", 0)
	Spring.SetGameRulesParam("killed_wisps", 0)

	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
		if Spring.GetUnitRulesParam(unitID, "is_dying") == 1 then
			pending[unitID] = Spring.GetUnitRulesParam(unitID, "destroy_frame")
		end
	end
end

GG.KillWisp = KillWisp
GG.SaveWisp = SaveWisp

end
