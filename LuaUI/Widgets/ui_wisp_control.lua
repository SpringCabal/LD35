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
include('keysym.h.lua')
local UP = KEYSYMS.UP
local DOWN = KEYSYMS.DOWN
local LEFT = KEYSYMS.LEFT
local RIGHT = KEYSYMS.RIGHT
local W = KEYSYMS.W
local S = KEYSYMS.S
local A = KEYSYMS.A
local D = KEYSYMS.D
local SPACE = KEYSYMS.SPACE

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local height

local pressSpace = 1

local function getMouseCoordinate(mx,my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true)
    if not pos then return false end
	local x, y, z = pos[1], pos[2], pos[3]
-- 	if x<2048 or z<2048 or x>8192 or z>8192 then	
-- 		return false
-- 	end
	return x,y,z
end

local function MouseControl()
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	
	if lmb and mouseControl1 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('inc_heightmap|' .. x .. '|' .. y .. '|' .. z .. "|" .. height)
			return true
		else
			return false
		end
	elseif rmb and mouseControl3 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('dec_heightmap|' .. x .. '|' .. y .. '|' .. z .. "|" .. height)
			return true
		else
			return false
		end
	end
end

local function MovementControl()
	local x, z = 0, 0
	
	if Spring.GetKeyState(A) or Spring.GetKeyState(LEFT) then
		x = x - 1
	end
	if Spring.GetKeyState(D) or Spring.GetKeyState(RIGHT) then
		x = x + 1
	end
	if Spring.GetKeyState(W) or Spring.GetKeyState(UP) then
		z = z - 1
	end
	if Spring.GetKeyState(S) or Spring.GetKeyState(DOWN) then
		z = z + 1
	end
	
	Spring.SendLuaRulesMsg('movement|' .. x .. '|' .. z)
end

function widget:MousePress(mx, my, button)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if not Spring.IsAboveMiniMap(mx, my) then
		if button == 1 then
			if Spring.GetGameRulesParam("has_arms") == 1 and Spring.GetGameRulesParam("spiritMode") == 0 then
				local traceType, unitID = Spring.TraceScreenRay(mx, my)
				if traceType == "unit" and UnitDefs[Spring.GetUnitDefID(unitID)].name == "lever" then
					local x, _, z = Spring.GetUnitPosition(unitID)
					local wx, _, wz = Spring.GetUnitPosition(wispID)
					local dx, dz = x - wx, z - wz
					if dx * dx + dz * dz < 100*100 then
						Spring.SendLuaRulesMsg('pull_lever|' .. unitID)
						return true
					end
				end
			end
			local x,y,z = getMouseCoordinate(mx,my)
			height = Spring.GetGroundHeight(x, z)
			if (x) then
				Spring.SendLuaRulesMsg('inc_heightmap|' .. x .. '|' .. y .. '|' .. z .. "|" .. height)
				mouseControl1 = true
				return true
			else
				return false
			end
		elseif button == 3 then
			local x,y,z = getMouseCoordinate(mx,my)
			height = Spring.GetGroundHeight(x, z)
			if (x) then
				Spring.SendLuaRulesMsg('dec_heightmap|' .. x .. '|' .. y .. '|' .. z .. "|" .. height)
				mouseControl3 = true
				return true
			else
				return false
			end
		end
	end	
end

function widget:GameFrame()
	if not wispID then
		return
	end

	if Spring.GetGameRulesParam("has_eyes") == 1 and pressSpace == 1 then
		pressSpace = 2
	end
	if pressSpace == 3 and Spring.GetGameRulesParam("game_end") == 1 then
		pressSpace = 4
	end
	if pressSpace == 4 and Spring.GetGameRulesParam("game_end") == 0 and Spring.GetGameRulesParam("has_eyes") == 0 then
		pressSpace = 1
	end
	
	if keyControl then
		MovementControl()
	end
	if mouseControl1 or mouseControl3 then
		MouseControl()
	end
end

function widget:DrawScreen()
	if pressSpace == 2 then
		gl.PushMatrix()
			gl.Text("Press Space", vsx * 0.4, vsy * 0.4, 20)
		gl.PopMatrix()
	end
end

function widget:MouseRelease(mx, my, button)
	if button == 1 then
		mouseControl1 = false
	elseif button == 3 then
		mouseControl3 = false
	end
end

-- handles weapon switching and abilities
function widget:KeyPress(key, mods, isRepeat)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	if wispID then
		if key == LEFT or key == RIGHT or key == UP or key == DOWN or key == W or key == A or key == S or key == D then
			keyControl = true
			return true
		end
		if key == SPACE then
			Spring.SendLuaRulesMsg('switch_form')
			pressSpace = 3
		end
	end
	
end

function widget:KeyRelease(key)
	if not (Spring.GetKeyState(A) or Spring.GetKeyState(LEFT) or Spring.GetKeyState(D) or Spring.GetKeyState(RIGHT) or Spring.GetKeyState(W) or Spring.GetKeyState(UP) or Spring.GetKeyState(S) or Spring.GetKeyState(DOWN)) then
		keyControl = false
	end
end

-- Custom lighting
local function GetMouseLight(beamLights, beamLightCount, pointLights, pointLightCount)
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	local x,y,z = getMouseCoordinate(mx,my)

	if x then
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 100, pz = z, param = {r = 0.9, g = 0.9, b = 0.9, radius = 5000}, colMult = 1}
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

local function GetWispLight(beamLights, beamLightCount, pointLights, pointLightCount)
	if wispID and Spring.GetGameRulesParam("spiritMode") == 1 then
		local x, y, z = Spring.GetUnitPosition(wispID)
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 50, pz = z, param = {r = 4, g = 4, b = 4, radius = 2000}, colMult = 1}
	end

	return beamLights, beamLightCount, pointLights, pointLightCount
end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()
	
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
	
	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_RegisterFunction(GetMouseLight)
		WG.DeferredLighting_RegisterFunction(GetWispLight)
	end
end

-- function widget:Initialize()
-- -- 	if WG.DeferredLighting_RegisterFunction then
-- -- 		WG.DeferredLighting_RegisterFunction(GetMouseLight)
-- -- 		WG.DeferredLighting_RegisterFunction(GetWispLight)
-- -- 	end
-- end
