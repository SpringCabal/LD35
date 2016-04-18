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

local idMapping = {}

-- trigger -> triggerable
local simpleTriggerMapping = {
	-- room 1 safe levers
	[14725] = 23261,
	[15262] = 27501,
	[22482] = 5316,
	[27806] = 15054,
	-- room 1 kill
	[26492] = { -23261, -27501, -5316, -15054 },
	
	-- ears room
	[31488] = { -41222, -41225 },
	[12701] = { 41222, 41225 },
	[17231] = 30700,
	
	-- level + door to right part of dungeon
	[4949] = 27633,
	
	[12688] = { -1811, -16824, -24489, -28033},
	[13688] = { -1811, -16824, -24489, -28033},
	[14688] = { -1811, -16824, -24489, -28033},
	[15688] = { -1811, -16824, -24489, -28033},
	
	[18496] = { 1811, 16824, 24489, 28033},
	[11688] = 2833,
}

-- trigerrable -> {trigger bitmap} 
local bitmapTriggerMapping = {
	-- levers + door to ears room
	[30165] = {29318, -23884, 14619, 19115}
}

function gadget:GameFrame()
	if GG.s11n ~= nil and not loaded then
		loaded = true
		
		for _, unitID in pairs(Spring.GetAllUnits()) do
			local unitDef = UnitDefs[Spring.GetUnitDefID(unitID)]
			if not unitDef.customParams.dungeonelement then
				Spring.DestroyUnit(unitID)
			end
		end

		local elements = VFS.LoadFile(customElements)
		elements = loadstring(elements)()
		local unitBridge = GG.s11n:GetUnitBridge()
		for id, unit in pairs(elements.units) do
			local unitID = unitBridge:Add(unit)
			idMapping[id] = unitID
		end
		
		for idTrigger, idTriggerables in pairs(simpleTriggerMapping) do
			local triggerID = idMapping[idTrigger]
			if type(idTriggerables) ~= "table" then
				idTriggerables = { idTriggerables }
			end
			for _, idTriggerable in pairs(idTriggerables) do
				local reverseEff = false
				if idTriggerable < 0 then
					reverseEff = true
					idTriggerable = math.abs(idTriggerable)
				end
				local triggerableID = idMapping[idTriggerable]
				if not reverseEff then
					GG.Plate.SimpleLink(triggerID, triggerableID)
				else
					GG.Plate.SimpleLink(triggerID, triggerableID, { do_kill = 1 })
				end
			end
		end
	end
end
