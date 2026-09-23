//! The player wisp: movement, body-part collection and piece visibility.
//!
//! Ported from `unit_wisp_controller.lua`.

use alloc::format;
use alloc::vec::Vec;

use partsneeded_shared::{BodyPart, BodyParts};

use crate::state::{Wisp, param};

/// How far ahead of the wisp the move goal is placed.  A long lookahead has to
/// be clamped against the map edge, which rotates diagonal input onto an axis;
/// keep it short, the goal is re-issued every frame anyway.
const MOVE_LOOKAHEAD: f32 = 120.0;
// Keep the same playable bounds as the original Lua controller.  A larger
// artificial margin makes the authored start position (2000, 2000) sit on the
// movement boundary on this map, so one half of WASD appears completely dead.
const MAP_EDGE_CLEARANCE: f32 = 1.0;
/// unit_powerup_controller.lua picks a part up within 50 elmos.
const PICKUP_RADIUS: f32 = 50.0;

/// Shorten an offset until it lands inside the map, preserving its heading.
/// Clamping x and z independently would turn a diagonal into a cardinal.
fn clamp_offset_to_map(origin: spring::Float3, dx: f32, dz: f32) -> (f32, f32) {
    let Ok([points_x, points_z]) = spring::get_height_map_size() else {
        return (origin.x + dx, origin.z + dz);
    };
    // get_height_map_size() reports heightmap points (mapx + 1), not elmos.
    const SQUARE_SIZE: f32 = 8.0;
    let max_x = (points_x - 1) as f32 * SQUARE_SIZE - MAP_EDGE_CLEARANCE;
    let max_z = (points_z - 1) as f32 * SQUARE_SIZE - MAP_EDGE_CLEARANCE;
    let mut scale: f32 = 1.0;
    let mut limit = |start: f32, delta: f32, hi: f32| {
        if delta > 0.0 && start + delta > hi {
            scale = scale.min((hi - start) / delta);
        } else if delta < 0.0 && start + delta < MAP_EDGE_CLEARANCE {
            scale = scale.min((MAP_EDGE_CLEARANCE - start) / delta);
        }
    };
    limit(origin.x, dx, max_x);
    limit(origin.z, dz, max_z);
    let scale = scale.max(0.0);
    (origin.x + dx * scale, origin.z + dz * scale)
}

pub fn move_wisp(wisp: &Wisp) -> bool {
    let Some((x, z)) = wisp.movement else {
        return false;
    };
    let Ok(position) = spring::get_unit_position(wisp.id, false, false) else {
        return false;
    };
    let length = (x * x + z * z).sqrt();
    if length == 0.0 {
        return false;
    }
    let (goal_x, goal_z) = clamp_offset_to_map(
        position,
        x / length * MOVE_LOOKAHEAD,
        z / length * MOVE_LOOKAHEAD,
    );
    let goal = spring::Float3::new(goal_x, position.y, goal_z);
    // Raw goal: head straight where the player pointed instead of letting the
    // pathfinder plan a detour around whatever is in the way.
    spring::set_unit_move_goal(wisp.id, goal, 16.0, 0.0, true).unwrap_or(false)
}

/// Ground-unit collision resolution can leave a positive vertical velocity at
/// sharp wall corners. The authored Lua controller explicitly removed that
/// component every frame; keep the same safety net in the WASM controller so a
/// corner cannot turn a normal stop into a launch over the dungeon.
pub fn cancel_upward_velocity(wisp: &Wisp) {
    if wisp.id < 0 {
        return;
    }
    let Ok(velocity) = spring::get_unit_velocity(wisp.id) else {
        return;
    };
    if velocity.y > 0.0 {
        let _ = spring::unit_control::set_unit_velocity(
            wisp.id,
            spring::Float3::new(velocity.x, 0.0, velocity.z),
        );
    }
}

pub fn collect_body_parts(wisp: &mut Wisp) -> Vec<(i32, bool, i32)> {
    let Ok(position) = spring::get_unit_position(wisp.id, false, false) else {
        return Vec::new();
    };
    let mut collected = Vec::new();
    for id in &wisp.body_parts {
        let Ok(part_position) = spring::get_unit_position(*id, false, false) else {
            continue;
        };
        let dx = position.x - part_position.x;
        let dz = position.z - part_position.z;
        if dx * dx + dz * dz > PICKUP_RADIUS * PICKUP_RADIUS {
            continue;
        }
        let Some(part) = spring::get_unit_def_id(*id)
            .ok()
            .and_then(|def| body_part(def.0))
        else {
            continue;
        };
        param(&format!("has_{}", part.name()), 1.0);
        collected.push((*id, part));
    }
    for &(_, part) in &collected {
        if !wisp.collected_parts.contains(part) {
            wisp.collected_parts.insert(part);
            wisp.pieces_dirty = true;
        }
    }
    wisp.body_parts
        .retain(|id| !collected.iter().any(|(collected, _)| collected == id));
    collected
        .into_iter()
        // Reclaimed: a pickup vanishes at once, with no death animation.
        .map(|(id, _)| (id, true, wisp.id))
        .collect()
}

pub fn rotate_body_parts(wisp: &Wisp) {
    // Match unit_powerup_controller.lua: loose parts slowly rotate until
    // collected, making pickups readable against the dark level geometry.
    for id in &wisp.body_parts {
        let Ok(rotation) = spring::rules_synced::units_info::get_unit_rotation(*id) else {
            continue;
        };
        let _ = spring::set_unit_rotation(
            *id,
            spring::Float3::new(rotation.pitch, rotation.yaw + 0.1, rotation.roll),
        );
    }
}

pub fn body_part(def: i32) -> Option<BodyPart> {
    BodyPart::ALL
        .into_iter()
        .find(|part| spring::get_unit_def_id_by_name(part.name()).ok() == Some(def))
}

/// wisp.lua's piece groups: the body part that shows each piece.  Body and
/// Head (`None`) show whenever the wisp is embodied; pieces not listed here,
/// the hair, never do.
pub const WISP_PIECE_GROUPS: [(&str, Option<BodyPart>); 18] = [
    ("Body", None),
    ("Head", None),
    ("Eyes", Some(BodyPart::Eyes)),
    ("Ears", Some(BodyPart::Ears)),
    ("ArmL", Some(BodyPart::Arms)),
    ("ArmR", Some(BodyPart::Arms)),
    ("ForeArmL", Some(BodyPart::Arms)),
    ("ForeArmR", Some(BodyPart::Arms)),
    ("HandL", Some(BodyPart::Arms)),
    ("HandR", Some(BodyPart::Arms)),
    ("ThighL", Some(BodyPart::Legs)),
    ("ThighR", Some(BodyPart::Legs)),
    ("CrusL", Some(BodyPart::Legs)),
    ("CrusR", Some(BodyPart::Legs)),
    ("FootL", Some(BodyPart::Legs)),
    ("FootR", Some(BodyPart::Legs)),
    ("Nose", Some(BodyPart::Nose)),
    ("Mouth", Some(BodyPart::Mouth)),
];

/// Whether a group's pieces show for a wisp embodied with `parts`
/// (`None`: a spirit, with every piece hidden).
pub fn group_visible(group: Option<BodyPart>, parts: Option<BodyParts>) -> bool {
    parts.is_some_and(|parts| group.is_none_or(|part| parts.contains(part)))
}

pub fn hide_all_wisp_pieces(id: i32) {
    crate::unit_scripts::queue_piece_mask(id, None);
}

pub fn sync_wisp_pieces(id: i32, spirit: bool, collected_parts: BodyParts) {
    if id < 0 {
        return;
    }
    // The unit script owns the piece groups and the gait they select.
    let parts = (!spirit).then_some(collected_parts);
    crate::unit_scripts::queue_piece_mask(id, parts);
    // Lua's Show(parent) semantics differ on the Core renderer: making the
    // body visible can implicitly re-enable children that Lua had hidden.
    // Apply the exact authored mask at the engine piece level as well.
    if let Ok(pieces) = spring::get_unit_piece_map(id) {
        for piece in pieces {
            let visible = WISP_PIECE_GROUPS
                .iter()
                .find(|(name, _)| *name == piece.name)
                .is_some_and(|(_, group)| group_visible(*group, parts));
            // GetUnitPieceMap exposes Lua's one-based piece numbers; the Core
            // visibility call is zero-based.
            let _ = spring::unit_control::set_unit_piece_visible(id, piece.piece_num - 1, visible);
        }
    }
}
