//! Levers, plates and the doors they drive.

use crate::cmd;
use crate::state::{Global, PUBLIC, Pending, World};

/// The four gate rectangles from map_customs.lua.  These are terrain walls,
/// not just decoration: the authored Lua game raises the height map while a
/// gate is closed, so a wisp cannot walk through the doorway.
#[allow(clippy::excessive_precision, clippy::inconsistent_digit_grouping)]
const GATE_AREAS: &[(i32, f32, f32, f32, f32)] = &[
    (20, 4105.493_16, 1250.606_93, 4340.235_35, 1300.816_77),
    (21, 5100.146_97, 1518.489_01, 5158.723_14, 1784.553_59),
    (22, 5968.859_86, 1843.365_36, 6152.629_39, 1920.571_29),
    (23, 6616.530_52, 1519.306_52, 6675.505_37, 1801.675_17),
];

const GATE_CLOSED_HEIGHT: f32 = 499.0;
const GATE_OPEN_HEIGHT: f32 = 411.5;

/// The standing plates, by level key.  A plate is pressed while an embodied
/// player with legs stands on it.
const PLATES: [i32; 8] = [26492, 31488, 12701, 12688, 13688, 14688, 15688, 11688];

/// The corridor gates.  Plates open them one way: they stay open once opened.
const ONE_WAY_GATES: [i32; 3] = [21, 22, 23];

/// Gate 20 opens from a lever combination instead of a link.
pub const LEVER_GATE: i32 = 20;

/// game_custom_spawn.lua: `["20"] = {-29318, 23884, -14619, -19115}`.
///
/// The sign is *not* "must be off".  A negative id sets `reverseEff`, which
/// BitmaskLink stores as the second tuple element, and unit_plate.lua then
/// fails the mask on `triggerObj[2] ~= active` -- so a negative id means the
/// lever must be ON, and the one positive id means it must be OFF.  The floor
/// markers in that room show the same three circles and one cross.
pub const LEVER_GATE_COMBINATION: [(i32, bool); 4] =
    [(29318, true), (23884, false), (14619, true), (19115, true)];

pub fn update_triggers(global: &Global) {
    // This is the Rust counterpart of `unit_plate.lua` and the link tables in
    // `game_custom_spawn.lua`.  The map IDs are stable content IDs; only the
    // engine-assigned unit IDs vary from run to run.
    let (has_legs, spirit, wisp_id) = {
        let wisp = global.wisp.access();
        (
            wisp.collected_parts.iter().any(|part| part == "legs"),
            wisp.spirit,
            wisp.id,
        )
    };
    let mut world = global.world.access_mut();
    let mut pending = global.pending.access_mut();
    for key in PLATES {
        let Some(plate) = level_id(&world, key) else {
            continue;
        };
        let active = has_legs && !spirit && wisp_id >= 0 && nearby(wisp_id, plate, 60.0);
        set_trigger(&mut world, plate, active);
    }
    for link in LINKS {
        let Some(source) = level_id(&world, link.source) else {
            continue;
        };
        let Some(target) = level_id(&world, link.target) else {
            continue;
        };
        // The corridor plates are one-way progression switches.  Their
        // authored Lua link opens the gate on the rising edge and does not
        // issue a closing command when the player walks away.  Keep the gate
        // open once it has been opened; the lever-controlled gates remain
        // reversible because gate 20 is handled by its bitmap below.
        let active = is_trigger_active(&world, source)
            || (ONE_WAY_GATES.contains(&link.target)
                && world.active_triggerables.contains(&target));
        set_triggerable_active(&mut world, &mut pending, target, active);
        if link.kill && active {
            let _ = spring::set_unit_rules_param_f32(target, "do_kill", 1.0, PUBLIC);
        }
    }
    let Some(gate) = level_id(&world, LEVER_GATE) else {
        return;
    };
    let bitmap_open = LEVER_GATE_COMBINATION.into_iter().all(|(key, expected)| {
        level_id(&world, key).is_some_and(|id| is_trigger_active(&world, id) == expected)
    });
    set_triggerable_active(&mut world, &mut pending, gate, bitmap_open);
}

/// Apply the initial closed state after the level units have been created.
/// Map gates can predate the WASM rules module, so this also covers adopted
/// map units rather than relying on UnitCreated.
pub fn initialize_gate_terrain(world: &World) {
    for (key, x1, z1, x2, z2) in GATE_AREAS {
        let Some(id) = level_id(world, *key) else {
            continue;
        };
        set_gate_physics(id, *x1, *z1, *x2, *z2, false);
    }
}

pub fn level_id(world: &World, key: i32) -> Option<i32> {
    world
        .level_ids
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, id)| *id)
}

pub fn is_trigger_active(world: &World, id: i32) -> bool {
    world.active_triggers.contains(&id)
}

pub fn set_trigger(world: &mut World, id: i32, active: bool) {
    if active {
        if !is_trigger_active(world, id) {
            world.active_triggers.push(id);
        }
    } else {
        world.active_triggers.retain(|candidate| *candidate != id);
    }
}

pub fn set_triggerable_active(world: &mut World, pending: &mut Pending, id: i32, active: bool) {
    let was_active = world.active_triggerables.contains(&id);
    if was_active == active {
        return;
    }
    if active {
        world.active_triggerables.push(id);
    } else {
        world
            .active_triggerables
            .retain(|candidate| *candidate != id);
    }
    if let Some((_, x1, z1, x2, z2)) = gate_area(world, id) {
        // Unlike cages, gates are reversible: send both edges so the unit
        // script and the terrain collision stay in sync.
        set_gate_physics(id, x1, z1, x2, z2, active);
        let _ = spring::give_order_to_unit(id, cmd::ONOFF, &[if active { 1.0 } else { 0.0 }], 0, 0);
    } else if active {
        // Cages still only receive their rising edge; replaying their release
        // animation every frame would re-trigger the prisoner lifecycle.
        let _ = spring::give_order_to_unit(id, cmd::ONOFF, &[1.0], 0, 0);
        if spring::get_unit_def_id_by_name("cage")
            .ok()
            .is_some_and(|cage| {
                spring::get_unit_def_id(id)
                    .ok()
                    .is_some_and(|def| def.0 == cage)
            })
        {
            pending.cage_activations.push(id);
        }
    }
}

fn gate_area(world: &World, id: i32) -> Option<(i32, f32, f32, f32, f32)> {
    GATE_AREAS
        .iter()
        .copied()
        .find(|(key, _, _, _, _)| level_id(world, *key) == Some(id))
}

fn set_gate_physics(id: i32, x1: f32, z1: f32, x2: f32, z2: f32, open: bool) {
    let height = if open {
        GATE_OPEN_HEIGHT
    } else {
        GATE_CLOSED_HEIGHT
    };
    let _ = spring::level_height_map(x1, z1, x2, z2, height);
    // The terrain is the gate's collision. The model itself stays embedded in
    // the wall and must never become a second, offset blocker.
    crate::level::clear_blocking(id, false);
    let _ = spring::force_unit_collision_update(id);
}

pub fn nearby(first: i32, second: i32, distance: f32) -> bool {
    let (Ok(first), Ok(second)) = (
        spring::get_unit_position(first, false, false),
        spring::get_unit_position(second, false, false),
    ) else {
        return false;
    };
    let x = first.x - second.x;
    let z = first.z - second.z;
    x * x + z * z <= distance * distance
}

#[derive(Clone, Copy)]
struct Link {
    source: i32,
    target: i32,
    kill: bool,
}

const LINKS: &[Link] = &[
    Link {
        source: 14725,
        target: 23261,
        kill: false,
    },
    Link {
        source: 15262,
        target: 27501,
        kill: false,
    },
    Link {
        source: 22482,
        target: 5316,
        kill: false,
    },
    Link {
        source: 27806,
        target: 15054,
        kill: false,
    },
    Link {
        source: 26492,
        target: 23261,
        kill: true,
    },
    Link {
        source: 26492,
        target: 27501,
        kill: true,
    },
    Link {
        source: 26492,
        target: 5316,
        kill: true,
    },
    Link {
        source: 26492,
        target: 15054,
        kill: true,
    },
    Link {
        source: 31488,
        target: 41222,
        kill: true,
    },
    Link {
        source: 31488,
        target: 41225,
        kill: true,
    },
    Link {
        source: 12701,
        target: 41222,
        kill: false,
    },
    Link {
        source: 12701,
        target: 41225,
        kill: false,
    },
    Link {
        source: 17231,
        target: 22,
        kill: false,
    },
    Link {
        source: 4949,
        target: 21,
        kill: false,
    },
    Link {
        source: 12688,
        target: 1811,
        kill: true,
    },
    Link {
        source: 12688,
        target: 16824,
        kill: true,
    },
    Link {
        source: 12688,
        target: 24489,
        kill: true,
    },
    Link {
        source: 12688,
        target: 28033,
        kill: true,
    },
    Link {
        source: 13688,
        target: 1811,
        kill: true,
    },
    Link {
        source: 13688,
        target: 16824,
        kill: true,
    },
    Link {
        source: 13688,
        target: 24489,
        kill: true,
    },
    Link {
        source: 13688,
        target: 28033,
        kill: true,
    },
    Link {
        source: 14688,
        target: 1811,
        kill: true,
    },
    Link {
        source: 14688,
        target: 16824,
        kill: true,
    },
    Link {
        source: 14688,
        target: 24489,
        kill: true,
    },
    Link {
        source: 14688,
        target: 28033,
        kill: true,
    },
    Link {
        source: 15688,
        target: 1811,
        kill: true,
    },
    Link {
        source: 15688,
        target: 16824,
        kill: true,
    },
    Link {
        source: 15688,
        target: 24489,
        kill: true,
    },
    Link {
        source: 15688,
        target: 28033,
        kill: true,
    },
    Link {
        source: 18496,
        target: 1811,
        kill: false,
    },
    Link {
        source: 18496,
        target: 16824,
        kill: false,
    },
    Link {
        source: 18496,
        target: 24489,
        kill: false,
    },
    Link {
        source: 18496,
        target: 28033,
        kill: false,
    },
    Link {
        source: 11688,
        target: 23,
        kill: false,
    },
];
