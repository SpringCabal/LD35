--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Cages",
		desc	= "Kills/frees wisps in cages and spawns them",
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
local cageDefID = UnitDefNames["cage"].id

local toCreate = {}

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if cageDefID == unitDefID then
		toCreate[unitID] = unitID
	end
end

function gadget:GameFrame()
	for _, unitID in pairs(toCreate) do
		local x, y, z = Spring.GetUnitPosition(unitID)
		Spring.CreateUnit(npcWispDefID, x, y, z, 0, 0)
	end
	toCreate = {}
end

end
