--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Game Custom spawn",
		desc      = "Spawns custom game elements",
		author    = "gajop",
		date      = "April 2016",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true  --  loaded by default?
	}
end

local customElements = "LuaRules/Configs/map_customs.lua"
local loaded = false


function gadget:GameFrame()
	if GG.s11n ~= nil and not loaded then
		Spring.SendCommands({"globallos"})
		loaded = true

		local elements = VFS.LoadFile(customElements)
		elements = loadstring(elements)()
		local unitBridge = GG.s11n:GetUnitBridge()
		for _, unit in pairs(elements.units) do
			unitBridge:Add(unit)
		end
	end
end
