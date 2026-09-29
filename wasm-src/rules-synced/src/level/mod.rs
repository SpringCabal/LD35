use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::{Float3, ResultExt, commands};

use crate::log::check;

mod units;

use units::{GATES, LEVEL_UNITS};

/// The dungeon floor height every authored unit stands on.
pub const FLOOR_Y: f32 = 411.499_02;

/// Take a unit out of the collision world.  Levers keep `quad_map_rays` so a
/// screen trace can still find them to click.
pub fn clear_blocking(id: i32, quad_map_rays: bool) {
    let result = spring::set_unit_blocking(
        id,
        spring::rules_synced::unit_control::SetUnitBlockingOptions {
            blocking: false,
            solid_objects: false,
            projectiles: false,
            quad_map_rays,
            crushable: false,
            block_enemy_pushing: false,
            block_height_changes: false,
        },
    );
    result.log_err_once("taking a unit out of the collision world");
}

#[derive(Clone, Copy)]
pub struct LevelUnit {
    pub key: i32,
    pub name: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Euler rotation from `map_customs.lua`, applied with SetUnitRotation the
    /// way s11n's unit bridge does.
    pub rot: spring::Float3,
    /// Mid/aim position offsets, relative to the unit, when the entry carries a
    /// `midAimPos` override.
    ///
    /// These are not cosmetic.  A cage's collision volume centres on its mid
    /// position, and the authored offset pushes it 31 elmos back into the cell;
    /// without it the volume sits on the model origin, right where unit_cage.lua
    /// spawns the prisoner, and the engine shoves the new wisp out through the
    /// bars to the nearest free spot.
    pub mid: Option<spring::Float3>,
    pub aim: Option<spring::Float3>,
}

/// Create a unit of the player's team at `position`; `then` gets its id once
/// it exists, after its UnitCreated.
pub fn spawn_unit(
    name: &'static str,
    def: i32,
    position: Float3,
    then: impl FnOnce(i32) + 'static,
) {
    let options = spring::CreateUnitOptions {
        build: false,
        flatten_ground: false,
        unit_id: -1,
        builder_id: -1,
    };
    commands::create_unit(spring::UnitDefRef { name, id: def }, position, 0, options).then(
        move |created| match created {
            Ok(id) => then(id),
            Err(error) => spring::log_warning!(
                crate::log::SECTION,
                "creating {name} at ({:.0}, {:.0}) failed: {error:?}",
                position.x,
                position.z
            ),
        },
    );
}

/// Engine unit id by map_customs.lua content key, filled in as the level's
/// units are created.
type Ids = Rc<RefCell<Vec<(i32, i32)>>>;

fn create(unit: LevelUnit, ids: &Ids) {
    let Some(def) = crate::defs::defs().by_name(unit.name) else {
        spring::log_warning!(crate::log::SECTION, "unknown level unit {}", unit.name);
        return;
    };
    let ids = Rc::clone(ids);
    spawn_unit(
        unit.name,
        def,
        Float3::new(unit.x, unit.y, unit.z),
        move |id| {
            configure(&unit, def, id);
            ids.borrow_mut().push((unit.key, id));
        },
    );
}

/// Runs as soon as the unit exists, so its calls apply in order: the gate
/// setup reads back the position set here.
fn configure(unit: &LevelUnit, def: i32, id: i32) {
    let defs = crate::defs::defs();
    // All authored static level geometry is placed at an explicit map height.
    // The player wisp is also an authored entry, but it must retain its normal
    // ground move type; putting it under MoveCtrl makes the walk animation run
    // while the unit itself remains completely immobile.
    if def != defs.wisp {
        configure_fixed_height(id, Float3::new(unit.x, unit.y, unit.z));
    }
    // game_custom_spawn.lua restores these through s11n's unit bridge, which
    // replays every serialised field, rotation and mid/aim positions included.
    check(
        "level unit rotation",
        id,
        spring::set_unit_rotation(id, unit.rot),
    );
    if let (Some(mid), Some(aim)) = (unit.mid, unit.aim) {
        check(
            "level unit mid and aim position",
            id,
            spring::set_unit_mid_and_aim_pos(id, mid, aim, true),
        );
    }
    if def == defs.gate {
        configure_gate(id, unit.y);
    }
}

fn configure_fixed_height(unit_id: i32, position: Float3) {
    check(
        "MoveCtrl",
        unit_id,
        spring::move_ctrl::move_ctrl(unit_id, true),
    );
    check(
        "MoveCtrl gravity",
        unit_id,
        spring::move_ctrl::set_move_ctrl_gravity(unit_id, 0.0),
    );
    check(
        "level unit position",
        unit_id,
        spring::set_unit_position(unit_id, position),
    );
}

/// Gates are terrain doors, not free-standing blockers. Keep their model at
/// the floor height that was authored for the map while the closed terrain is
/// raised around it. MoveCtrl prevents the engine from lifting the model onto
/// that raised terrain, matching Gravitas' wall setup.
fn configure_gate(unit_id: i32, floor_y: f32) {
    check(
        "gate always visible",
        unit_id,
        spring::set_unit_always_visible(unit_id, true),
    );
    check(
        "gate neutral",
        unit_id,
        spring::set_unit_neutral(unit_id, true),
    );
    check(
        "gate MoveCtrl",
        unit_id,
        spring::move_ctrl::move_ctrl(unit_id, true),
    );
    check(
        "gate MoveCtrl no blocking",
        unit_id,
        spring::move_ctrl::set_no_blocking(unit_id, true),
    );
    check(
        "gate MoveCtrl gravity",
        unit_id,
        spring::move_ctrl::set_move_ctrl_gravity(unit_id, 0.0),
    );
    if let Some(position) = check(
        "gate position",
        unit_id,
        spring::get_unit_position(unit_id, false, false),
    ) {
        check(
            "gate floor height",
            unit_id,
            spring::set_unit_position(
                unit_id,
                spring::Float3::new(position.x, floor_y, position.z),
            ),
        );
    }
    clear_blocking(unit_id, false);
}

/// Claim the four gates the map's featureplacer has already built.
///
/// game_custom_spawn.lua destroyed the map's gates and rebuilt them, purely so
/// they were registered with s11n.  The map's own gates are already in the
/// right places facing the right way, so adopt them and only build one if the
/// map somehow did not.
fn adopt_map_gates(ids: &Ids) {
    // A gate slot is only ever matched by a gate standing essentially on it.
    const MATCH_RADIUS_SQUARED: f32 = 64.0 * 64.0;
    let mut claimed = [false; GATES.len()];
    let mut adopted = Vec::new();

    let gate_def = crate::defs::defs().gate;
    if let Ok(units) = spring::get_all_units() {
        for unit_id in units {
            if crate::defs::unit_def(unit_id) != gate_def {
                continue;
            }
            let Ok(position) = spring::get_unit_position(unit_id, false, false) else {
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

    // Set up in queue order: after the level units created above.
    let adopted_ids = Rc::clone(ids);
    commands::run(move || {
        for (slot, unit_id) in adopted {
            configure_gate(unit_id, GATES[slot].y);
            adopted_ids.borrow_mut().push((GATES[slot].key, unit_id));
        }
    });
    for (slot, gate) in GATES.iter().enumerate() {
        if !claimed[slot] {
            create(*gate, ids);
        }
    }
}

/// Build the level. `done` gets each unit's engine id by content key once
/// every unit exists and is configured.
pub fn spawn(done: impl FnOnce(Vec<(i32, i32)>) + 'static) {
    let ids: Ids = Rc::new(RefCell::new(Vec::with_capacity(
        LEVEL_UNITS.len() + GATES.len(),
    )));
    for unit in LEVEL_UNITS {
        create(unit, &ids);
    }
    adopt_map_gates(&ids);
    commands::run(move || done(ids.take()));
}
