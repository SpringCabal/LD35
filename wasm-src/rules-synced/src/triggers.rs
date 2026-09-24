//! Levers, plates and the doors they drive.

use partsneeded_shared::BodyPart;
use spring::cmd;

use alloc::vec::Vec;

use crate::state::{Global, PUBLIC, World};

/// The four gate rectangles from map_customs.lua.  These are terrain walls,
/// not just decoration: the authored Lua game raises the height map while a
/// gate is closed, so a wisp cannot walk through the doorway.
#[allow(clippy::excessive_precision, clippy::inconsistent_digit_grouping)]
pub(crate) const GATE_AREAS: &[(i32, f32, f32, f32, f32)] = &[
    (20, 4105.493_16, 1250.606_93, 4340.235_35, 1300.816_77),
    (21, 5100.146_97, 1518.489_01, 5158.723_14, 1784.553_59),
    (22, 5968.859_86, 1843.365_36, 6152.629_39, 1920.571_29),
    (23, 6616.530_52, 1519.306_52, 6675.505_37, 1801.675_17),
];

pub(crate) const GATE_CLOSED_HEIGHT: f32 = 499.0;
const GATE_OPEN_HEIGHT: f32 = 411.5;

/// The standing plates, by level key.  A plate is pressed once an embodied
/// player with legs stands on it.
const PLATES: [i32; 8] = [26492, 31488, 12701, 12688, 13688, 14688, 15688, 11688];

/// Gate 20 opens from a lever combination instead of a link.
pub const LEVER_GATE: i32 = 20;

/// game_custom_spawn.lua: `["20"] = {-29318, 23884, -14619, -19115}`.
///
/// The sign is *not* "must be off".  A negative id sets `reverseEff`, which
/// BitmaskLink stores as the second tuple element, and unit_plate.lua then
/// fails the mask on `triggerObj[2] ~= active` -- so a negative id means the
/// lever must be ON, and the one positive id means it must be OFF.  The floor
/// markers in that room show the same three circles and one cross.
const LEVER_GATE_COMBINATION: [(i32, bool); 4] =
    [(29318, true), (23884, false), (14619, true), (19115, true)];

/// unit_plate.lua's radius: a plate is pressed by a player standing this close.
const PLATE_RADIUS: f32 = 60.0;

/// One pass of unit_plate.lua's GameFrame.  Returns the cages it opened,
/// each with whether it kills its prisoner.
///
/// unit_plate.lua only ever sends ONOFF 1 (its ONOFF 0 is commented out), so
/// everything but a lever latches: a pressed plate stays down, and a door or
/// cage stays open once any of its sources has opened it.  Levers are the
/// only toggles; the player flips them (`input::pull_lever`).
///
/// cage.lua reads `do_kill` when the cage activates, so the first link that
/// opens a cage decides its prisoner's fate: a lever link ahead of a kill
/// link in the same pass saves it, and a kill link set later changes nothing.
pub fn update_triggers(global: &Global) -> Vec<(i32, bool)> {
    let (has_legs, spirit, wisp_id) = {
        let wisp = global.wisp.access();
        (
            wisp.collected_parts.contains(BodyPart::Legs),
            wisp.spirit,
            wisp.id,
        )
    };
    // The ONOFF orders go out after the world borrow ends: an order can raise
    // AllowCommand and other callins synchronously.
    let (plates, targets, cages) = {
        let mut world = global.world.access_mut();
        let standing = has_legs && !spirit && wisp_id >= 0;
        let mut plates = Vec::new();
        for key in PLATES {
            let Some(plate) = level_id(&world, key) else {
                continue;
            };
            if standing && !is_trigger_active(&world, plate) && nearby(wisp_id, plate, PLATE_RADIUS)
            {
                set_trigger(&mut world, plate, true);
                plates.push(plate);
            }
        }
        let mut targets = Vec::new();
        let mut cages = Vec::new();
        for link in LINKS {
            let (Some(source), Some(target)) =
                (level_id(&world, link.source), level_id(&world, link.target))
            else {
                continue;
            };
            if !is_trigger_active(&world, source) {
                continue;
            }
            if link.kill {
                crate::log::check(
                    "do_kill rules param",
                    target,
                    spring::set_unit_rules_param_f32(target, "do_kill", 1.0, PUBLIC),
                );
            }
            if switch_on(&mut world, target) {
                targets.push((target, gate_area(&world, target)));
                if crate::defs::unit_def(target) == crate::defs::defs().cage {
                    cages.push((target, crate::cages::kills(target)));
                }
            }
        }
        if let Some(gate) = level_id(&world, LEVER_GATE) {
            let open = LEVER_GATE_COMBINATION.into_iter().all(|(key, expected)| {
                level_id(&world, key).is_some_and(|id| is_trigger_active(&world, id) == expected)
            });
            if open && switch_on(&mut world, gate) {
                targets.push((gate, gate_area(&world, gate)));
            }
        }
        (plates, targets, cages)
    };
    for plate in plates {
        give_onoff(plate, true);
    }
    for (id, gate) in targets {
        if let Some((_, x1, z1, x2, z2)) = gate {
            set_gate_physics(id, x1, z1, x2, z2, true);
        }
        give_onoff(id, true);
    }
    cages
}

fn give_onoff(id: i32, on: bool) {
    crate::log::check(
        "ONOFF order",
        id,
        spring::give_order_to_unit(id, cmd::ONOFF, &[if on { 1.0 } else { 0.0 }], 0, 0),
    );
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
    world.level_ids.get(&key).copied()
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

/// Turn a triggerable on; true if it was off.
fn switch_on(world: &mut World, id: i32) -> bool {
    if world.active_triggerables.contains(&id) {
        return false;
    }
    world.active_triggerables.push(id);
    true
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
    // The terrain is what actually blocks a closed doorway.
    crate::log::check(
        "gate terrain",
        id,
        spring::level_height_map(x1, z1, x2, z2, height),
    );
    // The terrain is the gate's collision. The model itself stays embedded in
    // the wall and must never become a second, offset blocker.
    crate::level::clear_blocking(id, false);
    crate::log::check(
        "gate collision update",
        id,
        spring::force_unit_collision_update(id),
    );
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
