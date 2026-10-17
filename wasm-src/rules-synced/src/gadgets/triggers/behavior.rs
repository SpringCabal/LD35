use partsneeded_shared::BodyPart;
use spring::cmd;

use alloc::vec::Vec;

use crate::state::{Global, PUBLIC, World};

#[allow(clippy::excessive_precision, clippy::inconsistent_digit_grouping)]
pub(crate) const GATE_AREAS: &[(i32, f32, f32, f32, f32)] = &[
    (20, 4105.493_16, 1250.606_93, 4340.235_35, 1300.816_77),
    (21, 5100.146_97, 1518.489_01, 5158.723_14, 1784.553_59),
    (22, 5968.859_86, 1843.365_36, 6152.629_39, 1920.571_29),
    (23, 6616.530_52, 1519.306_52, 6675.505_37, 1801.675_17),
];

pub(crate) const GATE_CLOSED_HEIGHT: f32 = 499.0;
const GATE_OPEN_HEIGHT: f32 = 411.5;

const PLATES: [i32; 8] = [26492, 31488, 12701, 12688, 13688, 14688, 15688, 11688];

pub const LEVER_GATE: i32 = 20;

const LEVER_GATE_COMBINATION: [(i32, bool); 4] =
    [(29318, true), (23884, false), (14619, true), (19115, true)];

const PLATE_RADIUS: f32 = 60.0;

pub fn update_triggers(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
) -> Vec<(i32, bool)> {
    let (has_legs, spirit, wisp_id) = {
        let wisp = global.wisp.access();
        (
            wisp.collected_parts.contains(BodyPart::Legs),
            wisp.spirit,
            wisp.id,
        )
    };
    let mut world = global.world.access_mut();
    let standing = has_legs && !spirit && wisp_id >= 0;
    for key in PLATES {
        let Some(plate) = level_id(&world, key) else {
            continue;
        };
        if standing
            && !is_trigger_active(&world, plate)
            && nearby(spring, wisp_id, plate, PLATE_RADIUS)
        {
            set_trigger(&mut world, plate, true);
            give_onoff(spring, plate, true);
        }
    }
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
                spring,
                "do_kill rules param",
                target,
                spring
                    .rules_params()
                    .set_unit_rules_param_f32(target, "do_kill", 1.0, PUBLIC),
            );
        }
        if switch_on(&mut world, target) {
            open(spring, &world, target);
            if crate::defs::unit_def(spring, target) == crate::defs::defs(spring).cage {
                cages.push((target, crate::cages::kills(spring, target)));
            }
        }
    }
    if let Some(gate) = level_id(&world, LEVER_GATE) {
        let open_gate = LEVER_GATE_COMBINATION.into_iter().all(|(key, expected)| {
            level_id(&world, key).is_some_and(|id| is_trigger_active(&world, id) == expected)
        });
        if open_gate && switch_on(&mut world, gate) {
            open(spring, &world, gate);
        }
    }
    cages
}

fn open(spring: &spring::Spring<'_, spring::RulesSynced>, world: &World, id: i32) {
    if let Some((_, x1, z1, x2, z2)) = gate_area(world, id) {
        set_gate_physics(spring, id, x1, z1, x2, z2, true);
    }
    give_onoff(spring, id, true);
}

fn give_onoff(_spring: &spring::Spring<'_, spring::RulesSynced>, id: i32, on: bool) {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .give_order_to_unit(id, cmd::ONOFF, &[if on { 1.0 } else { 0.0 }], 0, 0)
    })
    .then(move |spring, result| {
        crate::log::check(spring, "ONOFF order", id, result);
    });
}

pub fn initialize_gate_terrain(spring: &spring::Spring<'_, spring::RulesSynced>, world: &World) {
    for (key, x1, z1, x2, z2) in GATE_AREAS {
        let Some(id) = level_id(world, *key) else {
            continue;
        };
        set_gate_physics(spring, id, *x1, *z1, *x2, *z2, false);
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

fn set_gate_physics(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    id: i32,
    x1: f32,
    z1: f32,
    x2: f32,
    z2: f32,
    open: bool,
) {
    let height = if open {
        GATE_OPEN_HEIGHT
    } else {
        GATE_CLOSED_HEIGHT
    };
    crate::log::check(
        spring,
        "gate terrain",
        id,
        spring
            .terrain_control()
            .level_height_map(x1, z1, x2, z2, height),
    );
    crate::level::clear_blocking(spring, id, false);
    crate::log::check(
        spring,
        "gate collision update",
        id,
        spring.unit_control().force_unit_collision_update(id),
    );
}

pub fn nearby(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    first: i32,
    second: i32,
    distance: f32,
) -> bool {
    let (Ok(first), Ok(second)) = (
        spring
            .units_info()
            .get_unit_position(first, Default::default()),
        spring
            .units_info()
            .get_unit_position(second, Default::default()),
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
