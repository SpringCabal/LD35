local barsFront = piece("BarsFront")

function script.Create()
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

local npcWispDefID = UnitDefNames["npcwisp"].id

-- TODO: blocking/collision will be incosistent while it's pulling down/up
function script.Activate()
	local do_kill = Spring.GetUnitRulesParam(unitID, "do_kill") or 0
    StartThread(function()
        local x, y, z = Spring.GetUnitPosition(unitID)
		if do_kill ~= 1 then
			Spring.PlaySoundFile("sounds/cage.02.ogg", 1)
			Move(barsFront, z_axis, -200, 150)
			WaitForMove(barsFront, z_axis)
		end
			
		for _, npcWispID in pairs(Spring.GetUnitsInCylinder(x, z, 50)) do
			if Spring.GetUnitDefID(npcWispID) == npcWispDefID and Spring.GetUnitRulesParam(npcWispID, "is_dying") ~= 1 then
				if do_kill == 1 then
					GG.KillWisp(npcWispID)
				else
					GG.SaveWisp(npcWispID)
				end
			end
		end
--         Move(middle, z_axis, -5, 50);
    end)
    Spring.UnitScript.Signal(signalMask)
    return 1
end

function script.Deactivate()
    StartThread(function()
        local x, y, z = Spring.GetUnitPosition(unitID)
		Spring.PlaySoundFile("sounds/cage.02.ogg", 1)
		Move(barsFront, z_axis, 0, 150)
--         WaitForMove(middle, z_axis);
--         Move(middle, z_axis, 0, 50);
    end)
    Spring.UnitScript.Signal(signalMask)
    return 0
end
