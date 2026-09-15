use alloc::vec::Vec;

use spring::Float3;

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

fn create(unit: &LevelUnit) -> Option<i32> {
    let def = spring::get_unit_def_id_by_name(unit.name).ok()?;
    let id = spring::create_unit(
        spring::UnitDefRef {
            name: unit.name,
            id: def,
        },
        Float3::new(unit.x, unit.y, unit.z),
        0,
        spring::CreateUnitOptions {
            build: false,
            flatten_ground: false,
            unit_id: -1,
            builder_id: -1,
        },
    )
    .ok()?;
    // All authored static level geometry is placed at an explicit map height.
    // The player wisp is also an authored entry, but it must retain its normal
    // ground move type; putting it under MoveCtrl makes the walk animation run
    // while the unit itself remains completely immobile.
    if unit.name != "wisp" {
        configure_fixed_height(id, Float3::new(unit.x, unit.y, unit.z));
    }
    // game_custom_spawn.lua restores these through s11n's unit bridge, which
    // replays every serialised field.  This port only replayed the position, so
    // the six body-part pickups -- the only entries with a real rotation -- all
    // stood facing north.  The mid/aim entries turn out to match what the cage
    // and lever models already provide, so replaying them changes nothing
    // today; they are here because the bridge replays them and a future edit to
    // map_customs.lua should not silently stop working.
    let _ = spring::set_unit_rotation(id, unit.rot);
    if let (Some(mid), Some(aim)) = (unit.mid, unit.aim) {
        let _ = spring::set_unit_mid_and_aim_pos(id, mid, aim, true);
    }
    if unit.name == "gatesmoth" {
        configure_gate(id, unit.y);
    }
    Some(id)
}

fn configure_fixed_height(unit_id: i32, position: Float3) {
    let _ = spring::move_ctrl::move_ctrl(unit_id, true);
    let _ = spring::move_ctrl::set_move_ctrl_gravity(unit_id, 0.0);
    let _ = spring::set_unit_position(unit_id, position);
}

/// Gates are terrain doors, not free-standing blockers. Keep their model at
/// the floor height that was authored for the map while the closed terrain is
/// raised around it. MoveCtrl prevents the engine from lifting the model onto
/// that raised terrain, matching Gravitas' wall setup.
fn configure_gate(unit_id: i32, floor_y: f32) {
    let _ = spring::set_unit_always_visible(unit_id, true);
    let _ = spring::set_unit_neutral(unit_id, true);
    let _ = spring::move_ctrl::move_ctrl(unit_id, true);
    let _ = spring::move_ctrl::set_no_blocking(unit_id, true);
    let _ = spring::move_ctrl::set_move_ctrl_gravity(unit_id, 0.0);
    if let Ok(position) = spring::get_unit_position(unit_id, false, false) {
        let _ = spring::set_unit_position(
            unit_id,
            spring::Float3::new(position.x, floor_y, position.z),
        );
    }
    let _ = spring::set_unit_blocking(
        unit_id,
        spring::rules_synced::unit_control::SetUnitBlockingOptions {
            blocking: false,
            solid_objects: false,
            projectiles: false,
            quad_map_rays: false,
            crushable: false,
            block_enemy_pushing: false,
            block_height_changes: false,
        },
    );
}

/// Claim the four gates the map's featureplacer has already built.
///
/// game_custom_spawn.lua destroyed the map's gates and rebuilt them, purely so
/// they were registered with s11n.  Doing the rebuild half without the destroy
/// half left eight gates stacked in four places, and since the guest
/// create_unit() takes no facing argument the new ones all pointed north --
/// two gates crossed at ninety degrees.  The map's own gates are already in
/// the right places facing the right way, so adopt them and only build one if
/// the map somehow did not.
fn adopt_map_gates(ids: &mut Vec<(i32, i32)>) {
    // A gate slot is only ever matched by a gate standing essentially on it.
    const MATCH_RADIUS_SQUARED: f32 = 64.0 * 64.0;
    let mut claimed = [false; GATES.len()];

    if let (Ok(gate_def), Ok(units)) = (
        spring::get_unit_def_id_by_name("gatesmoth"),
        spring::get_all_units(),
    ) {
        for unit_id in units {
            if spring::get_unit_def_id(unit_id).ok() != Some(spring::DefId(gate_def)) {
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
                configure_gate(unit_id, GATES[slot].y);
                ids.push((GATES[slot].key, unit_id));
            }
        }
    }

    for (slot, gate) in GATES.iter().enumerate() {
        if let Some(id) = (!claimed[slot]).then(|| create(gate)).flatten() {
            ids.push((gate.key, id));
        }
    }
}

pub fn spawn() -> Vec<(i32, i32)> {
    let mut ids = Vec::with_capacity(LEVEL_UNITS.len() + GATES.len());
    for unit in LEVEL_UNITS.iter() {
        if let Some(id) = create(unit) {
            ids.push((unit.key, id));
        }
    }
    adopt_map_gates(&mut ids);
    ids
}

/// Only used if the map somehow did not place a gate; adopt_map_gates()
/// claims the map's own ones first.
const GATES: [LevelUnit; 4] = [
    LevelUnit {
        key: 20,
        name: "gatesmoth",
        x: 4216.0,
        y: 411.499_02,
        z: 1288.0,
        // Area 20: buildinglist in game_custom_spawn.lua faces this one west.
        rot: Float3::new(0.0, -core::f32::consts::FRAC_PI_2, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 21,
        name: "gatesmoth",
        x: 5112.0,
        y: 411.499_02,
        z: 1656.0,
        // Area 21: buildinglist in game_custom_spawn.lua faces this one north.
        rot: Float3::new(0.0, core::f32::consts::PI, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 22,
        name: "gatesmoth",
        x: 6056.0,
        y: 411.499_02,
        z: 1880.0,
        // Area 22: buildinglist in game_custom_spawn.lua faces this one east.
        rot: Float3::new(0.0, core::f32::consts::FRAC_PI_2, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 23,
        name: "gatesmoth",
        x: 6648.0,
        y: 411.499_02,
        z: 1656.0,
        // Area 23: buildinglist in game_custom_spawn.lua faces this one south.
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
];

const LEVEL_UNITS: [LevelUnit; 57] = [
    LevelUnit {
        key: 5316,
        name: "cage",
        x: 1275.9552,
        y: 411.499_02,
        z: 1600.0,
        rot: Float3::new(0.0, 0.00350089, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 14725,
        name: "lever",
        x: 1_030.373,
        y: 411.499_02,
        z: 1680.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 15054,
        name: "cage",
        x: 1_152.843_8,
        y: 411.499_02,
        z: 1600.0,
        rot: Float3::new(0.0, 0.01340784, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 15262,
        name: "lever",
        x: 924.014_2,
        y: 411.499_02,
        z: 1680.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 22482,
        name: "lever",
        x: 1_278.015_6,
        y: 411.499_02,
        z: 1680.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 23261,
        name: "cage",
        x: 1_027.989_7,
        y: 411.499_02,
        z: 1600.0,
        rot: Float3::new(0.0, 0.00377329, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 26492,
        name: "plate",
        x: 1_412.51,
        y: 411.499_02,
        z: 1600.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 27468,
        name: "eyes",
        x: 1_365.579_3,
        y: 411.499_02,
        z: 1_884.525_9,
        rot: Float3::new(0.0, 4.404_433, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 27501,
        name: "cage",
        x: 922.790_04,
        y: 411.499_02,
        z: 1600.0,
        rot: Float3::new(0.0, -0.0029519, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 27806,
        name: "lever",
        x: 1_147.571_2,
        y: 411.499_02,
        z: 1680.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 25577,
        name: "wisp",
        x: 1035.0,
        y: 411.499_02,
        z: 2702.0,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 31488,
        name: "plate",
        x: 4_432.63,
        y: 411.499_02,
        z: 1_130.768_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 12701,
        name: "plate",
        x: 3_927.563_2,
        y: 411.499_02,
        z: 677.590_45,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 41222,
        name: "cage",
        x: 4_432.63,
        y: 411.499_02,
        z: 677.590_45,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 41225,
        name: "cage",
        x: 4_332.63,
        y: 411.499_02,
        z: 677.590_45,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 27489,
        name: "ears",
        x: 4_193.289,
        y: 411.499_02,
        z: 892.573_24,
        rot: Float3::new(0.0, 3.646_026_6, 0.0),
        mid: Some(Float3::new(15.781_738, 3.492_553_7, -35.369_75)),
        aim: Some(Float3::new(15.781_738, 3.492_553_7, -35.369_75)),
    },
    LevelUnit {
        key: 17231,
        name: "lever",
        x: 3_927.563_2,
        y: 411.499_02,
        z: 1_130.768_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 14619,
        name: "lever",
        x: 3_895.823_5,
        y: 411.499_02,
        z: 1_418.919_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 25.361_938)),
        aim: Some(Float3::new(0.0, 0.0, 25.361_938)),
    },
    LevelUnit {
        key: 19115,
        name: "lever",
        x: 4_026.267_8,
        y: 411.499_02,
        z: 1_418.919_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 25.361_938)),
        aim: Some(Float3::new(0.0, 0.0, 25.361_938)),
    },
    LevelUnit {
        key: 23884,
        name: "lever",
        x: 3_778.625_2,
        y: 411.499_02,
        z: 1_418.919_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 25.361_938)),
        aim: Some(Float3::new(0.0, 0.0, 25.361_938)),
    },
    LevelUnit {
        key: 29318,
        name: "lever",
        x: 3_672.266_4,
        y: 411.499_02,
        z: 1_418.919_1,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 25.361_938)),
        aim: Some(Float3::new(0.0, 0.0, 25.361_938)),
    },
    LevelUnit {
        key: 22389,
        name: "arms",
        x: 4_613.602,
        y: 411.499_02,
        z: 2_416.733,
        rot: Float3::new(0.0, 4.3707943, 0.0),
        mid: Some(Float3::new(0.92871094, -17.895233, 0.4140625)),
        aim: Some(Float3::new(0.92871094, -17.895233, 0.4140625)),
    },
    LevelUnit {
        key: 4736,
        name: "nose",
        x: 3_715.350_3,
        y: 411.499_02,
        z: 1_738.564_8,
        rot: Float3::new(0.0, 3.070_795_5, 0.0),
        mid: Some(Float3::new(27.486084, 0.0, 38.741333)),
        aim: Some(Float3::new(27.486084, 0.0, 38.741333)),
    },
    LevelUnit {
        key: 4949,
        name: "lever",
        x: 5_025.723_6,
        y: 411.499_02,
        z: 1_822.392_6,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 25.361_938)),
        aim: Some(Float3::new(0.0, 0.0, 25.361_938)),
    },
    LevelUnit {
        key: 1811,
        name: "cage",
        x: 5_387.999_5,
        y: 411.499_02,
        z: 1_562.534_5,
        rot: Float3::new(0.0, 0.01340784, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 31.452_637)),
        aim: Some(Float3::new(0.0, 0.0, 31.452_637)),
    },
    LevelUnit {
        key: 16824,
        name: "cage",
        x: 5_511.111_3,
        y: 411.499_02,
        z: 1_562.534_5,
        rot: Float3::new(0.0, 0.00350089, 0.0),
        mid: Some(Float3::new(-0.0004883, 0.0, 31.452_637)),
        aim: Some(Float3::new(-0.0004883, 0.0, 31.452_637)),
    },
    LevelUnit {
        key: 24489,
        name: "cage",
        x: 5_767.009_3,
        y: 411.499_02,
        z: 1_563.320_7,
        rot: Float3::new(0.0, 0.00350089, 0.0),
        mid: Some(Float3::new(-0.0004883, 0.0, 31.452_637)),
        aim: Some(Float3::new(-0.0004883, 0.0, 31.452_637)),
    },
    LevelUnit {
        key: 28033,
        name: "cage",
        x: 5_643.897_5,
        y: 411.499_02,
        z: 1_563.320_7,
        rot: Float3::new(0.0, 0.01340784, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 31.452_637)),
        aim: Some(Float3::new(0.0, 0.0, 31.452_637)),
    },
    LevelUnit {
        key: 18744,
        name: "mouth",
        x: 7_342.055_7,
        y: 411.499_02,
        z: 1676.7019,
        rot: Float3::new(0.0, -0.4123872, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 30249,
        name: "legs",
        x: 6_045.265_6,
        y: 411.499_02,
        z: 2_287.661_6,
        rot: Float3::new(0.0, 3.846_026_4, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 11688,
        name: "plate",
        x: 6_498.078_6,
        y: 411.499_02,
        z: 1_729.915_6,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 12688,
        name: "plate",
        x: 6_267.009_3,
        y: 411.499_02,
        z: 1_563.320_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 1.169_067_4)),
        aim: Some(Float3::new(0.0, 0.0, 1.169_067_4)),
    },
    LevelUnit {
        key: 13688,
        name: "plate",
        x: 6_267.009_3,
        y: 411.499_02,
        z: 1_623.320_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(0.0, 0.0, 1.169_067_4)),
        aim: Some(Float3::new(0.0, 0.0, 1.169_067_4)),
    },
    LevelUnit {
        key: 14688,
        name: "plate",
        x: 6_267.009_3,
        y: 411.499_02,
        z: 1_683.320_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 15688,
        name: "plate",
        x: 6_267.009_3,
        y: 411.499_02,
        z: 1_743.320_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 18496,
        name: "lever",
        x: 6_498.078_6,
        y: 411.499_02,
        z: 1_574.503_9,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 1851,
        name: "light",
        x: 778.067_44,
        y: 411.499_02,
        z: 1_503.559_3,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 1855,
        name: "light",
        x: 3600.7666,
        y: 411.499_02,
        z: 2_205.129_4,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 2930,
        name: "light",
        x: 7_544.940_4,
        y: 411.499_02,
        z: 2_106.08,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 5452,
        name: "light",
        x: 6_398.831_5,
        y: 411.499_02,
        z: 2_285.619_4,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 5631,
        name: "light",
        x: 4_852.581_5,
        y: 411.499_02,
        z: 2_307.896_5,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 6996,
        name: "light",
        x: 788.211_8,
        y: 411.499_02,
        z: 2_995.721_2,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 9635,
        name: "light",
        x: 2_113.023_4,
        y: 411.499_02,
        z: 2_098.526,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 10957,
        name: "light",
        x: 6_814.913,
        y: 411.499_02,
        z: 1373.3252,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 11015,
        name: "light",
        x: 1_337.520_6,
        y: 411.499_02,
        z: 2_455.557_9,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_444_3, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_444_3, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 12250,
        name: "light",
        x: 5_818.216,
        y: 411.499_02,
        z: 1_571.747_3,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 14408,
        name: "light",
        x: 6_727.247,
        y: 411.499_02,
        z: 1831.5614,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 15186,
        name: "light",
        x: 4_156.744,
        y: 411.499_02,
        z: 637.748_35,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 15522,
        name: "light",
        x: 5_821.177_7,
        y: 411.499_02,
        z: 2_066.409_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 18063,
        name: "light",
        x: 695.380_6,
        y: 411.499_02,
        z: 1_957.773_4,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: None,
        aim: None,
    },
    LevelUnit {
        key: 21184,
        name: "light",
        x: 5_315.296_4,
        y: 411.499_02,
        z: 1_564.179_9,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 21390,
        name: "light",
        x: 2_650.122,
        y: 411.499_02,
        z: 1_668.126_3,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 24068,
        name: "light",
        x: 7_279.023_4,
        y: 411.499_02,
        z: 1_594.510_6,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 26920,
        name: "light",
        x: 8_027.109_4,
        y: 411.499_02,
        z: 1_366.45,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 30541,
        name: "light",
        x: 1_331.415_5,
        y: 411.499_02,
        z: 2_958.379_6,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_444_3, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_444_3, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 30680,
        name: "light",
        x: 2_059.028_3,
        y: 411.499_02,
        z: 1_385.688_7,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
    LevelUnit {
        key: 31541,
        name: "light",
        x: 4_584.03,
        y: 411.499_02,
        z: 2597.6394,
        rot: Float3::new(0.0, 0.0, 0.0),
        mid: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
        aim: Some(Float3::new(3.580_566_4, 5.192_260_7, 19.247_07)),
    },
];
