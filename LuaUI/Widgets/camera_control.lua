--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
	return {
		name      = "Camera control",
		desc      = "Controls camera zooming and panning",
		author    = "gajop",
		date      = "WIP",
		license   = "GPLv2",
		version   = "0.1",
		layer     = -1000,
		enabled   = true,  --  loaded by default?
		handler   = true,
		api       = true,
		hidden    = true,
	}
end

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

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

local gameMode
function SetGameMode(gameMode)
	if Spring.GetGameRulesParam("gameMode") ~= "develop" then
		s = {
			dist = 518.541626,
			px = 1078.2821,
			py = 436.300781,
			pz = 2710.06079,
			rz = 0,
			dx = 0,
			dy = -0.8283768,
			dz = -0.5601712,
			fov = 45,
			ry = 0.01,
			mode = 2,
			rx = 2.54700017,
			name = "spring",
		}
		Spring.SetCameraState(s, 0)
	end
end
--------------------------------------------------------------------------------
-- Follow camera
--
-- The engine's unit tracker ("track") is not smoothed: for non-FPS camera modes
-- it does a bare SetPos() onto the unit's draw position every frame, so every
-- bit of jitter in the wisp's movement lands in the camera 1:1.  Re-issuing
-- "trackoff"/"track" each Update on top of that made it worse.  Follow the wisp
-- ourselves with exponential smoothing instead, driven off the frame delta so
-- the camera behaves the same at 55fps as at 150fps.

local FOLLOW_TAU = 0.12 -- seconds to close ~63% of the distance to the target
local LOOK_AHEAD = 8    -- frames of velocity to lead the wisp by

local camX, camY, camZ

local function UpdateCamera(dt)
	local x, y, z = Spring.GetUnitViewPosition(wispID)
	if not x then
		return
	end

	-- Lead the wisp slightly so the camera hints at where you are heading.
	local vx, vy, vz = Spring.GetUnitVelocity(wispID)
	if vx then
		x, y, z = x + vx*LOOK_AHEAD, y + vy*LOOK_AHEAD, z + vz*LOOK_AHEAD
	end

	if camX and dt and dt > 0 then
		local alpha = 1 - math.exp(-dt / FOLLOW_TAU)
		camX = camX + (x - camX)*alpha
		camY = camY + (y - camY)*alpha
		camZ = camZ + (z - camZ)*alpha
	else
		camX, camY, camZ = x, y, z
	end

	-- Transition time 0: the smoothing above is the only smoothing we want,
	-- stacking the engine's on top of it just adds lag.
	Spring.SetCameraTarget(camX, camY, camZ, 0)
end

function widget:Update(dt)
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and wispID ~= nil then
		UpdateCamera(dt)
	end
	local newGameMode = Spring.GetGameRulesParam("gameMode")
    if gameMode ~= newGameMode then
        gameMode = newGameMode
        camX, camY, camZ = nil, nil, nil
        SetGameMode(gameMode)
    end
end

function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
    for k, v in pairs(Spring.GetCameraState()) do
       print(k .. " = " .. tostring(v) .. ",")
    end

    gameMode = Spring.GetGameRulesParam("gameMode")
    SetGameMode(gameMode)
end

function widget:Shutdown()
end

function widget:MouseWheel(up,value)
    -- uncomment this to disable zoom/panning
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and wispID ~= nil then
		return true
	end
end
