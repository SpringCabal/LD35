local lever = piece('Base_001')

function script.Create()
	Turn(lever, x_axis, 0.6, 0);
end

function script.Killed(recentDamage, _)
    return 1
end

local signalMask = 0

function script.Activate()
    return 1
end

function script.Deactivate()
    return 0
end


-- TODO: blocking/collision will be incosistent while it's pulling down/up
function script.Activate()
    StartThread(function()
        local x, y, z = Spring.GetUnitPosition(unitID)
        Spring.PlaySoundFile("sounds/lever.ogg", 0.6)
        Turn(lever, x_axis, -0.6, 15);
--         WaitForMove(middle, z_axis);
--         Move(middle, z_axis, -5, 50);
    end)
    Spring.UnitScript.Signal(signalMask)
    return 1
end

function script.Deactivate()
    StartThread(function()
        local x, y, z = Spring.GetUnitPosition(unitID)
		Spring.PlaySoundFile("sounds/lever.ogg", 0.6)
        Turn(lever, x_axis, 0.6, 15);
--         WaitForMove(middle, z_axis);
--         Move(middle, z_axis, 0, 50);
    end)
    Spring.UnitScript.Signal(signalMask)
    return 0
end
