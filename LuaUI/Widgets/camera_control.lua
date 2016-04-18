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
frist = true
lastTrackingUpdate = os.clock()
function widget:Update()
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and wispID ~= nil then
		Spring.SelectUnitArray({wispID})
		Spring.SendCommands({"trackoff", "track"})
        if(frist) then
            Spring.SendCommands({"trackmode 1"});
            frist = false
        end
		Spring.SelectUnitArray({})
	end
	local newGameMode = Spring.GetGameRulesParam("gameMode")
    if gameMode ~= newGameMode then
        gameMode = newGameMode
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
