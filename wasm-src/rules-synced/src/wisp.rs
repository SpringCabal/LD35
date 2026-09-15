//! The player wisp: movement, body-part collection and piece visibility.
//!
//! Ported from `unit_wisp_controller.lua`.

use alloc::string::String;
use alloc::vec::Vec;

use crate::state::{Wisp, param};

/// How far ahead of the wisp the move goal is placed.  A long lookahead has to
/// be clamped against the map edge, which rotates diagonal input onto an axis;
/// keep it short, the goal is re-issued every frame anyway.
const MOVE_LOOKAHEAD: f32 = 120.0;
// Keep the same playable bounds as the original Lua controller.  A larger
// artificial margin makes the authored start position (2000, 2000) sit on the
// movement boundary on this map, so one half of WASD appears completely dead.
const MAP_EDGE_CLEARANCE: f32 = 1.0;

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
    // The loop reads `body_parts`, so the newly collected names are staged
    // here and merged after it ends.
    let mut collected_names: Vec<String> = Vec::new();
    let mut pieces_dirty = false;
    let mut has_eyes = false;
    for id in &wisp.body_parts {
        let Ok(part_position) = spring::get_unit_position(*id, false, false) else {
            continue;
        };
        let dx = position.x - part_position.x;
        let dz = position.z - part_position.z;
        if dx * dx + dz * dz > 2500.0 {
            continue;
        }
        let Ok(def) = spring::get_unit_def_id(*id) else {
            continue;
        };
        let Some(name) = body_part_name(def.0) else {
            continue;
        };
        let mut key = String::from("has_");
        key.push_str(name);
        param(&key, 1.0);
        if !wisp.collected_parts.iter().any(|part| part == name)
            && !collected_names.iter().any(|part| part == name)
        {
            collected_names.push(String::from(name));
            pieces_dirty = true;
        }
        if name == "eyes" {
            has_eyes = true;
        }
        collected.push(*id);
    }
    wisp.collected_parts.append(&mut collected_names);
    wisp.pieces_dirty |= pieces_dirty;
    wisp.has_eyes |= has_eyes;
    wisp.body_parts.retain(|id| !collected.contains(id));
    collected
        .into_iter()
        // The original trigger script's Killed callback is a no-op.  Core
        // does not run that legacy callback, so normal destruction would keep
        // its model alive while waiting for a script completion that cannot
        // arrive.  Reclaim removal gives the intended immediate pickup.
        .map(|id| (id, true, wisp.id))
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

pub fn body_part_name(def: i32) -> Option<&'static str> {
    ["eyes", "ears", "arms", "legs", "nose", "mouth"]
        .into_iter()
        .find(|name| spring::get_unit_def_id_by_name(name).ok() == Some(def))
}

pub fn hide_all_wisp_pieces(id: i32) {
    let _ = spring::typed::call_unit_script(id, "SetWasmPieceMask", &[-1.0], 0);
}

pub fn sync_wisp_pieces(id: i32, spirit: bool, collected_parts: &[String]) {
    if id < 0 {
        return;
    }
    // Preserve the original wisp.lua piece groups and animation state. This
    // is deliberately a unit-script call, not a renderer approximation.
    let mask = if spirit {
        -1
    } else {
        ["eyes", "ears", "arms", "legs", "nose", "mouth"]
            .into_iter()
            .enumerate()
            .filter(|(_, part)| collected_parts.iter().any(|owned| owned == *part))
            .fold(0, |mask, (bit, _)| mask | (1 << bit))
    };
    let _ = spring::typed::call_unit_script(id, "SetWasmPieceMask", &[mask as f32], 0);
    // Lua's Show(parent) semantics differ on the Core renderer: making the
    // body visible can implicitly re-enable children that Lua had hidden.
    // Apply the exact authored mask at the engine piece level as well, while
    // leaving the original unit script responsible for animation state.
    if let Ok(pieces) = spring::get_unit_piece_map(id) {
        for piece in pieces {
            let visible = !spirit
                && match piece.name.as_str() {
                    "Body" | "Head" => true,
                    "Eyes" => mask & 1 != 0,
                    "Ears" => mask & 2 != 0,
                    "ArmL" | "ArmR" | "ForeArmL" | "ForeArmR" | "HandL" | "HandR" => mask & 4 != 0,
                    "ThighL" | "ThighR" | "CrusL" | "CrusR" | "FootL" | "FootR" => mask & 8 != 0,
                    "Nose" => mask & 16 != 0,
                    "Mouth" => mask & 32 != 0,
                    _ => false,
                };
            // GetUnitPieceMap exposes Lua's one-based piece numbers; the Core
            // visibility call is zero-based.
            let _ = spring::unit_control::set_unit_piece_visible(id, piece.piece_num - 1, visible);
        }
    }
}

pub fn body_part_mask(wisp: &Wisp) -> u8 {
    wisp.collected_parts.iter().fold(0, |mask, part| {
        mask | match part.as_str() {
            "eyes" => 1,
            "ears" => 2,
            "arms" => 4,
            "legs" => 8,
            "nose" => 16,
            "mouth" => 32,
            _ => 0,
        }
    })
}
