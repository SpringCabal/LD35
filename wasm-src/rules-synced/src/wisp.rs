//! The player wisp: movement, body-part collection and piece visibility.
//!
//! Ported from `unit_wisp_controller.lua`.

use alloc::vec::Vec;

use partsneeded_shared::{BodyPart, BodyParts};
use springcabal_control::goal::{Bounds, Clamp, GoalHeight, MoveGoal};

use crate::state::Wisp;

/// Where the held keys send the wisp: a raw goal, straight where the player
/// pointed instead of a path finder's detour, at the wisp's own height.
///
/// A long lookahead has to be clamped against the map edge; keep it short,
/// the goal is re-issued every frame anyway. The edge clamp keeps the heading
/// (clamping x and z separately would turn a diagonal into a cardinal) and the
/// original controller's bounds: a larger margin puts the authored start
/// position (2000, 2000) on the boundary on this map, so one half of WASD
/// appears dead.
const STEER: MoveGoal = MoveGoal::ahead(120.0)
    .radius(16.0)
    .raw(true)
    .height(GoalHeight::Unit)
    .bounds(Bounds::Map {
        clearance: 1.0,
        clamp: Clamp::KeepHeading,
    });

/// unit_powerup_controller.lua picks a part up within 50 elmos.
const PICKUP_RADIUS: f32 = 50.0;

pub fn move_wisp(wisp: &Wisp) -> bool {
    let Some(direction) = wisp.steering.direction() else {
        return false;
    };
    STEER.apply(wisp.id, direction).unwrap_or(false)
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
    let defs = crate::defs::defs();
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
        let Some(part) = defs.body_part(crate::defs::unit_def(*id)) else {
            continue;
        };
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

pub fn sync_wisp_pieces(id: i32, spirit: bool, collected_parts: BodyParts) {
    if id < 0 {
        return;
    }
    // The unit script owns the piece groups and the gait they select.  This
    // runs from the rules' own GameFrame, never inside a unit-script export,
    // so the module is free.
    let parts = (!spirit).then_some(collected_parts);
    if crate::with_cus_module(|module| module.set_piece_mask(id, parts)).is_none() {
        spring::log_warning!(
            crate::log::SECTION,
            "piece mask for unit {id}: unit scripts busy"
        );
    }
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
