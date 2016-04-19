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
	[17231] = "22",
	
	-- level + door to right part of dungeon
	[4949] = "21",
	
	[12688] = { -1811, -16824, -24489, -28033},
	[13688] = { -1811, -16824, -24489, -28033},
	[14688] = { -1811, -16824, -24489, -28033},
	[15688] = { -1811, -16824, -24489, -28033},
	
	[18496] = { 1811, 16824, 24489, 28033},
	[11688] = "23",
}

-- trigerrable -> {trigger bitmap} 
local bitmapTriggerMapping = {
	-- levers + door to ears room
	["20"] = {-29318, 23884, -14619, -19115}
}

local elements

function GetAreaDoor(areaStr)
	local area = elements.areas[tonumber(areaStr)]
	for _, uid in pairs(Spring.GetUnitsInRectangle(area[1]-100, area[2]-100, area[3]+100, area[4]+100)) do
		if UnitDefs[Spring.GetUnitDefID(uid)].name == "gatesmoth" then
			return uid
		end
	end
end

local buildinglist = {
	{ name = 'gatesmoth', x = 5112, z = 1656, rot = "north" },
	{ name = 'gatesmoth', x = 6648, z = 1656, rot = "south" },
	{ name = 'gatesmoth', x = 4216, z = 1288, rot = "west" },
	{ name = 'gatesmoth', x = 6056, z = 1880, rot = "east" },
}

function gadget:GameFrame()
	if GG.s11n ~= nil and not loaded then
		loaded = true
		
		local unitBridge = GG.s11n:GetUnitBridge()
		for _, unitID in pairs(Spring.GetAllUnits()) do
			local unitDef = UnitDefs[Spring.GetUnitDefID(unitID)]
			if unitDef.name == "gatesmoth" then
-- 				local unit = unitBridge:Get(unitID)
-- 				unit.unitID = nil
-- 				unit.rot = nil
-- 				unit.dir = unitBridge:Get(unitID, "dir")
  				Spring.DestroyUnit(unitID)
--  				local uid = unitBridge:Add(unit)
			elseif not unitDef.customParams.dungeonelement then
				Spring.DestroyUnit(unitID)
			end
		end
		local gaiaID = Spring.GetGaiaTeamID()
		local los_status = {los=true, prevLos=true, contRadar=true, radar=true}
		for i,bDef in pairs(buildinglist) do
			local flagID = Spring.CreateUnit(bDef.name, bDef.x, 0, bDef.z, bDef.rot, gaiaID)
			Spring.SetUnitNeutral(flagID,true)
			Spring.SetUnitLosState(flagID,0,los_status)
			Spring.SetUnitAlwaysVisible(flagID,true)
			Spring.SetUnitBlocking(flagID,true)
		end
		delayFrame = Spring.GetGameFrame() + 3
	end
	if delayFrame == Spring.GetGameFrame() then
		local unitBridge = GG.s11n:GetUnitBridge()
		delayFrame = nil
		elements = VFS.LoadFile(customElements)
		elements = loadstring(elements)()
		for id, unit in pairs(elements.units) do
			local unitID = unitBridge:Add(unit)
			idMapping[id] = unitID
		end
		
		for id, area in pairs(elements.areas) do
			if id >= 20 and id <= 23 then
				for _, unitID in pairs(Spring.GetUnitsInRectangle(area[1]-200, area[2]-200, area[3]+200, area[4]+200)) do
					if UnitDefs[Spring.GetUnitDefID(unitID)].name == "gatesmoth" then
						GG.AddAreaWall(unitID, area)
					end
				end
			end
		end
		
		for idTrigger, idTriggerables in pairs(simpleTriggerMapping) do
			local triggerID = idMapping[idTrigger]
			if type(idTriggerables) ~= "table" then
				idTriggerables = { idTriggerables }
			end
			for _, idTriggerable in pairs(idTriggerables) do
				local reverseEff = false
				local triggerableID
				if type(idTriggerable) == "number" then
					if idTriggerable < 0 then
						reverseEff = true
						idTriggerable = math.abs(idTriggerable)
					end
					triggerableID = idMapping[idTriggerable]
				else
					triggerableID = GetAreaDoor(idTriggerable)
				end
				if not triggerableID then
					Spring.Echo("HELP!", idTriggerable)
				end
				if not reverseEff then
					GG.Plate.SimpleLink(triggerID, triggerableID)
				else
					GG.Plate.SimpleLink(triggerID, triggerableID, { do_kill = 1 })
				end
			end
		end
		
		for idTriggerable, idTriggers in pairs(bitmapTriggerMapping) do
			local triggerableID = GetAreaDoor(idTriggerable)
			local triggerMask = {}
			for _, idTrigger in pairs(idTriggers) do
				local reverseEff = false
				local triggerableID
				if idTrigger < 0 then
					reverseEff = true
					idTrigger = math.abs(idTrigger)
				end
				local triggerID = idMapping[idTrigger]
				table.insert(triggerMask, {triggerID, reverseEff})
			end
			GG.Plate.BitmaskLink(triggerMask, triggerableID)
		end
	end
end
