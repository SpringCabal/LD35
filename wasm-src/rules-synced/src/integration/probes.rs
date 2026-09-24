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
/// The same for the legs: the foot follows the thigh and the shin.
pub(super) const LEG_PIECE: &str = "FootR";

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

/// Distance from one of the wisp's pieces to the body piece it hangs off.
///
/// Float_Body bobs the whole model continuously, so measuring a limb against
/// the unit centre cannot tell a stroke or a step from that idle bob.  The
/// limb animations rotate the limb relative to Body, so this only moves when
/// the limb itself is animating.
pub(super) fn offset_from_body(unit_id: i32, piece: &str) -> Option<f32> {
    let arm = spring::typed::unit_piece_position_by_name(unit_id, piece)?;
    let body = spring::typed::unit_piece_position_by_name(unit_id, "Body")?;
    let (dx, dy, dz) = (arm.x - body.x, arm.y - body.y, arm.z - body.z);
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

/// Scripts/gate.lua slides this piece 168 elmos along z when the door opens.
pub(super) const GATE_PIECE: &str = "Cube41_extract45_GATE";
/// Scripts/plate.lua sinks this piece 2 elmos when the plate is pressed.
pub(super) const PLATE_PIECE: &str = "Button";

/// How far a level unit's moving piece sits from the unit's own origin.
///
/// Measuring the piece is what proves the unit script ran: the Rust-side
/// trigger bookkeeping can say "open" while the script never did.
pub(super) fn piece_offset(state: &Snapshot<'_>, key: i32, piece_name: &str) -> Option<f32> {
    // The piece position is relative to the unit.
    let piece =
        spring::typed::unit_piece_position_by_name(level_id(state.world, key)?, piece_name)?;
    Some((piece.x * piece.x + piece.y * piece.y + piece.z * piece.z).sqrt())
}

pub(super) fn format_position(position: Option<spring::Float3>) -> String {
    match position {
        Some(position) => alloc::format!("({:.1}, {:.1})", position.x, position.z),
        None => String::from("unavailable"),
    }
}
