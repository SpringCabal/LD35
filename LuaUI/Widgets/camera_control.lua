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

function widget:Update()
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and wispID ~= nil then
		Spring.SelectUnitArray({wispID})
		Spring.SendCommands("track " .. tostring(wispID))
	end
end

function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
    --for k, v in pairs(Spring.GetCameraState()) do
    --    Spring.Echo(k .. " = " .. tostring(v) .. ",")
    --end

    s = {
        px = 3150,
        py = 102.34146118164,
        pz = 3480,
        mode = 1,
        flipped = -1,
        dy = -0.90149933099747,
        dz = -0.43356931209564,
        fov = 45,
        height = 3300,
        angle = 0.46399998664856,
        dx = 0,
        name = "spring",
    }
--     Spring.SetCameraState(s, 0)
end

function widget:Shutdown()
end

function widget:MouseWheel(up,value)
    -- uncomment this to disable zoom/panning
    --return true
end
