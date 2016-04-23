local gate = piece('Cube41_extract45_GATE')
-- local rails = piece('Rails');

function script.Create()
end

function script.Killed(recentDamage, _)
    return 1
end

local signalMask = 1

local collisionData
-- TODO: blocking/collision will be incosistent while it's pulling down/up
function script.Activate()
    Spring.UnitScript.Signal(signalMask)
    StartThread(function()
        Spring.UnitScript.SetSignalMask(signalMask)
        GG.AdjustWallTerrain(unitID, 0)
        Sleep(500)
        local x, y, z = Spring.GetUnitPosition(unitID)
        Spring.PlaySoundFile("sounds/gate.ogg", 40)
        Move(gate, z_axis, -168, 180);
--         WaitForMove(gate, z_axis);
    end)
    return 1
end

function script.Deactivate()
    Spring.UnitScript.Signal(signalMask)
    StartThread(function()
        Spring.UnitScript.SetSignalMask(signalMask)
        local x, y, z = Spring.GetUnitPosition(unitID)
        Spring.PlaySoundFile("sounds/gate.ogg", 5, x, y, z)
        Move(gate, z_axis, 0, 190);
        WaitForMove(gate, z_axis);
        Spring.SetUnitBlocking(unitID, true)
        GG.AdjustWallTerrain(unitID, 1)
    end)
    return 0
end
