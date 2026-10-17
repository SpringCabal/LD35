use alloc::string::String;
use alloc::vec::Vec;

use super::Snapshot;
use super::scenario::TEST_LEVER;
use crate::defs::{defs, unit_def};
use crate::gadgets::triggers::behavior::level_id;

pub(super) fn count_units(spring: &spring::Spring<'_, spring::RulesSynced>, def: i32) -> usize {
    let Ok(units) = spring.units().get_all_units() else {
        return usize::MAX;
    };
    units
        .into_iter()
        .filter(|id| unit_def(spring, *id) == def)
        .count()
}

pub(super) const ANIMATED_PIECE: &str = "ForeArmR";
pub(super) const LEG_PIECE: &str = "FootR";

pub(super) fn lever_piece_offset(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    state: &Snapshot<'_>,
) -> Option<f32> {
    let lever = level_id(state.world, TEST_LEVER)?;
    let piece_number = spring
        .units_pieces()
        .get_unit_piece_map(lever)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "Base_001")
        .map(|piece| piece.piece_num)?;
    let piece = spring
        .units_pieces()
        .get_unit_piece_pos_dir(lever, piece_number)
        .ok()?;
    Some(piece.direction.x + piece.direction.y * 2.0 + piece.direction.z * 3.0)
}

pub(super) fn offset_from_body(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    piece: &str,
) -> Option<f32> {
    let arm = spring
        .units_pieces()
        .position_by_name(unit_id, piece)
        .ok()
        .flatten()?;
    let body = spring
        .units_pieces()
        .position_by_name(unit_id, "Body")
        .ok()
        .flatten()?;
    let (dx, dy, dz) = (arm.x - body.x, arm.y - body.y, arm.z - body.z);
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

pub(super) fn worst_wisp_cage_distance(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    state: &Snapshot<'_>,
) -> f32 {
    let Ok(units) = spring.units().get_all_units() else {
        return f32::MAX;
    };
    let cages: Vec<spring::Float3> = units
        .into_iter()
        .filter(|id| unit_def(spring, *id) == defs(spring).cage)
        .filter_map(|id| {
            spring
                .units_info()
                .get_unit_position(id, Default::default())
                .ok()
        })
        .collect();
    if cages.is_empty() {
        return f32::MAX;
    }
    let mut worst = 0.0f32;
    for wisp in &state.world.npc_wisps {
        let Ok(position) = spring
            .units_info()
            .get_unit_position(*wisp, Default::default())
        else {
            continue;
        };
        let nearest = cages
            .iter()
            .map(|cage| {
                let (dx, dz) = (cage.x - position.x, cage.z - position.z);
                dx * dx + dz * dz
            })
            .fold(f32::MAX, f32::min)
            .sqrt();
        worst = worst.max(nearest);
    }
    worst
}

pub(super) const GATE_PIECE: &str = "Cube41_extract45_GATE";
pub(super) const PLATE_PIECE: &str = "Button";

pub(super) fn piece_offset(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    state: &Snapshot<'_>,
    key: i32,
    piece_name: &str,
) -> Option<f32> {
    let piece = spring
        .units_pieces()
        .position_by_name(level_id(state.world, key)?, piece_name)
        .ok()
        .flatten()?;
    Some((piece.x * piece.x + piece.y * piece.y + piece.z * piece.z).sqrt())
}

pub(super) fn format_position(position: Option<spring::Float3>) -> String {
    match position {
        Some(position) => alloc::format!("({:.1}, {:.1})", position.x, position.z),
        None => String::from("unavailable"),
    }
}
