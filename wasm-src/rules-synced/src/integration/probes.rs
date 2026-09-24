//! Measurements the integration checks take of the running game.

use alloc::string::String;
use alloc::vec::Vec;

use super::Snapshot;
use super::scenario::TEST_LEVER;
use crate::defs::{defs, unit_def};
use crate::triggers::level_id;

/// How many live units of a unit def exist right now.
pub(super) fn count_units(def: i32) -> usize {
    let Ok(units) = spring::get_all_units() else {
        return usize::MAX;
    };
    units.into_iter().filter(|id| unit_def(*id) == def).count()
}

/// A child piece Scripts/wisp.lua actually moves when the upper arm turns.
/// Measuring the ArmR pivot itself is unreliable: rotating a piece does not
/// move that piece's origin, while its forearm child visibly follows it.
pub(super) const ANIMATED_PIECE: &str = "ForeArmR";

/// The lever arm's facing.
///
/// Scripts/lever.lua Turns the piece about its own pivot, which leaves its
/// position unchanged -- only the direction moves, so that is what to sample.
pub(super) fn lever_piece_offset(state: &Snapshot<'_>) -> Option<f32> {
    let lever = level_id(state.world, TEST_LEVER)?;
    let piece_number = spring::get_unit_piece_map(lever)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "Base_001")
        .map(|piece| piece.piece_num)?;
    let piece = spring::owned::units_pieces::get_unit_piece_pos_dir(lever, piece_number).ok()?;
    Some(piece.direction.x + piece.direction.y * 2.0 + piece.direction.z * 3.0)
}

/// Distance from the animated arm piece to the body piece it hangs off.
///
/// Float_Body bobs the whole model continuously, so measuring the arm against
/// the unit centre cannot tell a swim stroke from that idle bob.  The arm
/// animations rotate the arm relative to Body, so this only moves when the arm
/// itself is animating.
pub(super) fn arm_offset_from_body(unit_id: i32) -> Option<f32> {
    let arm = spring::typed::unit_piece_position_by_name(unit_id, ANIMATED_PIECE)?;
    let body = spring::typed::unit_piece_position_by_name(unit_id, "Body")?;
    let (dx, dy, dz) = (arm.x - body.x, arm.y - body.y, arm.z - body.z);
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

/// Distance from the unit's own centre to its animated piece.
///
/// The piece query reports a world position, which moves whenever the unit
/// walks -- comparing that directly passed even with the unit scripts disabled.
/// Measuring against the unit's own position is invariant to both translation
/// and heading, so it only changes when the piece really is animating.
pub(super) fn piece_offset_from_unit(unit_id: i32) -> Option<f32> {
    let piece = spring::typed::unit_piece_position_by_name(unit_id, ANIMATED_PIECE)?;
    let unit = spring::get_unit_position(unit_id, false, false).ok()?;
    let (dx, dy, dz) = (piece.x - unit.x, piece.y - unit.y, piece.z - unit.z);
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

/// The furthest any NPC wisp sits from the cage nearest it.
///
/// unit_cage.lua spawns each one at its cage's exact position, so they should
/// all read essentially zero; a wisp loose on the floor is a prisoner that
/// escaped its cell before the player ever arrived.
pub(super) fn worst_wisp_cage_distance(state: &Snapshot<'_>) -> f32 {
    let Ok(units) = spring::get_all_units() else {
        return f32::MAX;
    };
    let cages: Vec<spring::Float3> = units
        .into_iter()
        .filter(|id| unit_def(*id) == defs().cage)
        .filter_map(|id| spring::get_unit_position(id, false, false).ok())
        .collect();
    if cages.is_empty() {
        return f32::MAX;
    }
    let mut worst = 0.0f32;
    for wisp in &state.world.npc_wisps {
        let Ok(position) = spring::get_unit_position(*wisp, false, false) else {
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

/// How far the gate's moving slab sits from the gate unit's own origin.
///
/// Scripts/gate.lua slides `Cube41_extract45_GATE` 168 elmos along z when the
/// bitmask link opens the door.  Measuring the piece is what proves the door
/// actually opened: the Rust-side trigger bookkeeping can say "open" while the
/// unit script never ran.
pub(super) fn gate_piece_offset(state: &Snapshot<'_>, key: i32) -> Option<f32> {
    let gate = level_id(state.world, key)?;
    let piece_number = spring::get_unit_piece_map(gate)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "Cube41_extract45_GATE")
        .map(|piece| piece.piece_num)?;
    let piece = spring::owned::units_pieces::get_unit_piece_pos_dir(gate, piece_number).ok()?;
    let unit = spring::get_unit_position(gate, false, false).ok()?;
    let (dx, dy, dz) = (
        piece.position.x - unit.x,
        piece.position.y - unit.y,
        piece.position.z - unit.z,
    );
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

pub(super) fn format_position(position: Option<spring::Float3>) -> String {
    match position {
        Some(position) => alloc::format!("({:.1}, {:.1})", position.x, position.z),
        None => String::from("unavailable"),
    }
}
