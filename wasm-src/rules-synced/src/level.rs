use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::{Float3, ResultExt};

use crate::log::check;

mod units;

use units::{GATES, LEVEL_UNITS};

pub const FLOOR_Y: f32 = 411.499_02;

pub fn clear_blocking(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    id: i32,
    quad_map_rays: bool,
) {
    let result = spring.unit_control().set_unit_blocking(
        id,
        spring::unit_control::SetUnitBlockingOptions {
            blocking: false,
            solid_objects: false,
            projectiles: false,
            quad_map_rays,
            crushable: false,
            block_enemy_pushing: false,
            block_height_changes: false,
        },
    );
    result.log_err_once(spring, "taking a unit out of the collision world");
}

#[derive(Clone, Copy)]
pub struct LevelUnit {
    pub key: i32,
    pub name: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rot: spring::Float3,
    pub mid: Option<spring::Float3>,
    pub aim: Option<spring::Float3>,
}

pub fn spawn_unit(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    name: &'static str,
    def: i32,
    position: Float3,
    then: impl FnOnce(&spring::Spring<'_, spring::RulesSynced>, i32) + 'static,
) {
    let options = spring::unit_control::CreateUnitOptions {
        build: false,
        flatten_ground: false,
        unit_id: -1,
        builder_id: -1,
    };
    crate::with_addon_context(|ctx| {
        ctx.commands().create_unit(
            &spring::unit_control::DefRef {
                name: name.into(),
                id: def,
            },
            position,
            0,
            0,
            options,
        )
    })
    .then(move |spring, created| match created {
        Ok(id) => then(spring, id),
        Err(error) => spring::log_warning!(
            spring,
            crate::log::SECTION,
            "creating {name} at ({:.0}, {:.0}) failed: {error:?}",
            position.x,
            position.z
        ),
    });
}

type Ids = Rc<RefCell<Vec<(i32, i32)>>>;

fn create(spring: &spring::Spring<'_, spring::RulesSynced>, unit: LevelUnit, ids: &Ids) {
    let Some(def) = crate::defs::defs(spring).by_name(unit.name) else {
        spring::log_warning!(
            spring,
            crate::log::SECTION,
            "unknown level unit {}",
            unit.name
        );
        return;
    };
    let ids = Rc::clone(ids);
    spawn_unit(
        spring,
        unit.name,
        def,
        Float3::new(unit.x, unit.y, unit.z),
        move |spring, id| {
            configure(spring, &unit, def, id);
            ids.borrow_mut().push((unit.key, id));
        },
    );
}

fn configure(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: &LevelUnit,
    def: i32,
    id: i32,
) {
    let defs = crate::defs::defs(spring);
    if def != defs.wisp {
        configure_fixed_height(spring, id, Float3::new(unit.x, unit.y, unit.z));
    }
    check(
        spring,
        "level unit rotation",
        id,
        spring.unit_control().set_unit_rotation(id, unit.rot),
    );
    if let (Some(mid), Some(aim)) = (unit.mid, unit.aim) {
        check(
            spring,
            "level unit mid and aim position",
            id,
            spring
                .unit_control()
                .set_unit_mid_and_aim_pos(id, mid, aim, true),
        );
    }
    if def == defs.gate {
        configure_gate(spring, id, unit.y);
    }
}

fn configure_fixed_height(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    position: Float3,
) {
    check(
        spring,
        "MoveCtrl",
        unit_id,
        spring.move_ctrl().move_ctrl(unit_id, true),
    );
    check(
        spring,
        "MoveCtrl gravity",
        unit_id,
        spring.move_ctrl().set_move_ctrl_gravity(unit_id, 0.0),
    );
    check(
        spring,
        "level unit position",
        unit_id,
        spring.unit_control().set_unit_position(unit_id, position),
    );
}

fn configure_gate(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, _floor_y: f32) {
    check(
        spring,
        "gate always visible",
        unit_id,
        spring.unit_control().set_unit_always_visible(unit_id, true),
    );
    check(
        spring,
        "gate neutral",
        unit_id,
        spring.unit_control().set_unit_neutral(unit_id, true),
    );
}

// Lua replaces the four map gates in its first GameFrame, before loading the
// authored objects three frames later. Preserve that creation/RNG/ID order.
pub fn spawn_map_gates(spring: &spring::Spring<'_, spring::RulesSynced>) {
    check(
        spring,
        "global level visibility",
        -1,
        spring.team_control().set_global_los(0, true),
    );
    let gaia = spring.game().get_gaia_team_id().unwrap_or(2);
    let def = crate::defs::defs(spring).gate;
    for (slot, facing) in [(1, 2), (3, 0), (0, 3), (2, 1)] {
        let gate = GATES[slot];
        crate::with_addon_context(|ctx| {
            ctx.commands().create_unit(
                &spring::unit_control::DefRef {
                    name: "gatesmoth".into(),
                    id: def,
                },
                Float3::new(gate.x, 0.0, gate.z),
                facing,
                gaia,
                spring::unit_control::CreateUnitOptions {
                    build: false,
                    flatten_ground: true,
                    unit_id: -1,
                    builder_id: -1,
                },
            )
        })
        .then(|spring, created| {
            if let Some(id) = check(spring, "initial gate", -1, created) {
                check(
                    spring,
                    "initial gate neutral",
                    id,
                    spring.unit_control().set_unit_neutral(id, true),
                );
                check(
                    spring,
                    "initial gate visibility",
                    id,
                    spring.unit_control().set_unit_always_visible(id, true),
                );
                check(
                    spring,
                    "initial gate LOS",
                    id,
                    spring.unit_control().set_unit_los_state(id, 0, 15),
                );
            }
        });
    }
}

fn adopt_map_gates(spring: &spring::Spring<'_, spring::RulesSynced>, ids: &Ids) {
    const MATCH_RADIUS_SQUARED: f32 = 64.0 * 64.0;
    let mut claimed = [false; GATES.len()];
    let mut adopted = Vec::new();

    let gate_def = crate::defs::defs(spring).gate;
    if let Ok(units) = spring.units().get_all_units() {
        for unit_id in units {
            if crate::defs::unit_def(spring, unit_id) != gate_def {
                continue;
            }
            let Ok(position) = spring
                .units_info()
                .get_unit_position(unit_id, Default::default())
            else {
                continue;
            };
            let nearest = GATES
                .iter()
                .enumerate()
                .filter(|(slot, _)| !claimed[*slot])
                .map(|(slot, gate)| {
                    let dx = gate.x - position.x;
                    let dz = gate.z - position.z;
                    (slot, dx * dx + dz * dz)
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((slot, _)) = nearest.filter(|(_, d)| *d <= MATCH_RADIUS_SQUARED) {
                claimed[slot] = true;
                adopted.push((slot, unit_id));
            }
        }
    }

    let adopted_ids = Rc::clone(ids);
    crate::with_addon_context(|ctx| {
        ctx.commands().run(move |spring| {
            for (slot, unit_id) in adopted {
                configure_gate(spring, unit_id, GATES[slot].y);
                adopted_ids.borrow_mut().push((GATES[slot].key, unit_id));
            }
        })
    });
    for (slot, gate) in GATES.iter().enumerate() {
        if !claimed[slot] {
            create(spring, *gate, ids);
        }
    }
}

pub fn spawn(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    done: impl FnOnce(&spring::Spring<'_, spring::RulesSynced>, Vec<(i32, i32)>) + 'static,
) {
    let ids: Ids = Rc::new(RefCell::new(Vec::with_capacity(
        LEVEL_UNITS.len() + GATES.len(),
    )));
    // The Lua loader uses the same explicit key order. Hash-table traversal
    // must not change engine unit IDs, cage births or synced RNG consumption.
    let mut units = LEVEL_UNITS.to_vec();
    units.sort_unstable_by_key(|unit| unit.key);
    for unit in units {
        create(spring, unit, &ids);
    }
    adopt_map_gates(spring, &ids);
    crate::with_addon_context(|ctx| ctx.commands().run(move |spring| done(spring, ids.take())));
}
