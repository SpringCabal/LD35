function widget:GetInfo()
	return {
		name 	= "Wisp control",
		desc	= "Sends wisp actions from LuaUI to LuaRules",
		author	= "gajop",
		date	= "16 April 2016",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------
local mouseControl1 = false
local mouseControl3 = false
-- local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local function getMouseCoordinate(mx,my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true)
    if not pos then return false end
	local x, y, z = pos[1], pos[2], pos[3]
-- 	if x<2048 or z<2048 or x>8192 or z>8192 then	
-- 		return false
-- 	end
	return x,y,z
end

local function WeaponControl()
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	
	if lmb and mouseControl1 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('inc_heightmap|' .. x .. '|' .. y .. '|' .. z )
			return true
		else
			return false
		end
	elseif rmb and mouseControl3 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('dec_heightmap|' .. x .. '|' .. y .. '|' .. z )
			return true
		else
			return false
		end
	end
end

function widget:MousePress(mx, my, button)
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if not Spring.IsAboveMiniMap(mx, my) then

		if button == 1 then
			local x,y,z = getMouseCoordinate(mx,my)
			if (x) then
				Spring.SendLuaRulesMsg('zap|' .. x .. '|' .. y .. '|' .. z )
				mouseControl1 = true
				return true
			else
				return false
			end
		elseif button == 3 then
			local x,y,z = getMouseCoordinate(mx,my)
			if (x) then
				Spring.SendLuaRulesMsg('zap|' .. x .. '|' .. y .. '|' .. z )
				mouseControl3 = true
				return true
			else
				return false
			end
		end
	end	
end

function widget:GameFrame()
	if mouseControl1 or mouseControl3 --[[and Spring.GetGameRulesParam("gameMode") ~= "develop" ]] then
		WeaponControl()
	end
end

function widget:MouseRelease(mx, my, button)
	if button == 1 then
		mouseControl1 = false
	elseif button == 3 then
		mouseControl3 = false
	end
end

-- Custom lighting
local function GetMouseLight(beamLights, beamLightCount, pointLights, pointLightCount)
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	local x,y,z = getMouseCoordinate(mx,my)

	if x then
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 100, pz = z, param = {r = 0.4, g = 0.4, b = 0.4, radius = 3000}, colMult = 1}
	end

	return beamLights, beamLightCount, pointLights, pointLightCount
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
	end
end

function widget:UnitDestroyed(unitID)
	if wispID == unitID then
		wispID = nil
	end
end
function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

local function GetWispLight(beamLights, beamLightCount, pointLights, pointLightCount)
	if wispID then
		local x, y, z = Spring.GetUnitPosition(wispID)
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 50, pz = z, param = {r = 0.1, g = 0.1, b = 1, radius = 1000}, colMult = 1}
	end

	return beamLights, beamLightCount, pointLights, pointLightCount
end

function widget:Initialize()
	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_RegisterFunction(GetMouseLight)
		WG.DeferredLighting_RegisterFunction(GetWispLight)
	end
end
