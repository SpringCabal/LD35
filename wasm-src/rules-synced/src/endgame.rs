//! End-of-game sequencing and NPC wisp fates.

use alloc::vec::Vec;

use crate::sound::play_unit_sound;
use crate::state::{ENDING_WISP_DESTROY_DELAY, Global, PUBLIC, WISP_DESTROY_DELAY, param};

pub fn update_game_end(global: &Global) {
    let (has_all_parts, already_over, alive_wisps, saved_wisps, killed_wisps, settling) = {
        let wisp = global.wisp.access();
        let endgame = global.endgame.access();
        let world = global.world.access();
        let all_parts = ["eyes", "ears", "arms", "legs", "nose", "mouth"]
            .into_iter()
            .all(|part| wisp.collected_parts.iter().any(|found| found == part));
        // A rescued NPC remains in npc_wisps until its authored 1.5 second
        // death/cleanup delay expires. If every remaining NPC is already in
        // that saved/dead lifecycle, wait for the counters to settle instead
        // of incorrectly selecting Ending #1 (Welcome back) while the saves
        // are still pending.
        let settling = !world.npc_wisps.is_empty()
            && world
                .npc_wisps
                .iter()
                .all(|id| endgame.dying_wisps.iter().any(|(dying, _, _)| dying == id));
        (
            all_parts,
            endgame.game_over,
            world.npc_wisps.len(),
            endgame.saved_wisps,
            endgame.killed_wisps,
            settling,
        )
    };
    if !has_all_parts || already_over || settling {
        return;
    }

    let over_type = game_over_type(alive_wisps, saved_wisps, killed_wisps) as f32;
    {
        let mut endgame = global.endgame.access_mut();
        // Keep the state transition one-shot if a nested callin reaches this
        // function after the snapshot above but before this write.
        if endgame.game_over {
            return;
        }
        endgame.game_over = true;
        endgame.ending_type = over_type as i32;
        // Only the souls-freed ending has a delayed visual sequence.  The
        // other three outcomes go straight to the end screen after the
        // screen-effect fade, so expose that sequence as complete immediately
        // through the same status path used by the WASM UI.
        endgame.sequence_complete = over_type != 1.0;
        if over_type == 1.0 {
            endgame.release_wisps = true;
        }
    }
    param("game_over", 1.0);
    param("game_over_type", over_type);
    param(
        "game_over_sequence",
        if over_type == 1.0 { 0.0 } else { 1.0 },
    );
    if over_type == 1.0 {
        let wisp_id = {
            let mut wisp = global.wisp.access_mut();
            wisp.spirit = true;
            wisp.movement = None;
            wisp.stop_issued = true;
            wisp.pieces_dirty = true;
            wisp.id
        };
        // Match game_end.lua: freeze the player at the ceremony while the
        // incoming spirits converge on it. Input is rejected once game_over
        // is set, but an already-issued move goal would otherwise keep it
        // drifting during the ending.
        let _ = spring::move_ctrl::move_ctrl(wisp_id, true);
        param("spiritMode", 1.0);
    }
}

pub(crate) fn game_over_type(alive_wisps: usize, saved_wisps: i32, killed_wisps: i32) -> i32 {
    match (alive_wisps == 0, saved_wisps > 0, killed_wisps > 0) {
        (false, false, false) => 0,
        (true, true, false) => 1,
        (true, false, true) => 2,
        _ => 3,
    }
}

/// Lua's game_end.lua emits 500 short-lived saved wisps. Although it declares
/// a ten-frame cooldown, its `spawnedTime` local is never updated, so the
/// effective behavior is one arrival per GameFrame.
pub const ENDING_SPAWN_AMOUNT: i32 = 500;

/// Reproduce the Lua ending's independent signed x/z offsets without relying
/// on a host RNG. Lua samples each axis in [-300, 300], then extends the sign
/// by another 100 elmos, producing independent ranges [-400, -100] or
/// [100, 400]. A stable hash keeps replays deterministic while retaining that
/// square, multi-directional distribution.
fn ending_offset(index: u32, salt: u32) -> f32 {
    let mut value = index
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(salt)
        .wrapping_add(0x7f4a_7c15);
    value ^= value >> 16;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 13;
    let magnitude = 100.0 + (value % 301) as f32;
    if value & 1 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

pub fn spawn_ending_wisp(global: &Global, _frame: i32) {
    let wisp_id = global.wisp.access().id;
    let Some(position) = spring::get_unit_position(wisp_id, false, false).ok() else {
        return;
    };
    let index = global.endgame.access().end_spawned.saturating_sub(1) as u32;
    let dx = ending_offset(index, 0x243f_6a88);
    let dz = ending_offset(index, 0x85a3_08d3);
    let spawn_position = spring::Float3::new(position.x + dx, position.y, position.z + dz);
    let Ok(def) = spring::get_unit_def_id_by_name("npcwisp") else {
        return;
    };
    let Ok(id) = spring::create_unit(
        spring::UnitDefRef {
            name: "npcwisp",
            id: def,
        },
        spawn_position,
        0,
        spring::CreateUnitOptions {
            build: false,
            flatten_ground: false,
            unit_id: -1,
            builder_id: -1,
        },
    ) else {
        return;
    };
    // Hide the physical wisp model before the next render pass. The UI draws
    // these as spirit billboards; leaving the default model visible for a
    // frame is the source of the brief corporal flashes in the old ending.
    crate::wisp::hide_all_wisp_pieces(id);
    // Match game_end.lua: let the engine's normal Wisp movement handle the
    // approach. The old WASM port MoveCtrl-teleported these one fixed step at
    // a time, which changed acceleration and made the stream visibly unlike
    // Lua. CMD_MOVE is the engine's stable numeric MOVE command.
    let _ = spring::give_order_to_unit(id, 10, &[position.x, position.y, position.z], 0, 0);

    // UnitCreated normally adds this NPC to the authoritative alive set. Add
    // it explicitly as well because the Core callout may be delivered after
    // create_unit returns; the contains check prevents a duplicate either way.
    let added_alive = {
        let mut world = global.world.access_mut();
        if !world.npc_wisps.contains(&id) {
            world.npc_wisps.push(id);
            Some(world.npc_wisps.len() as f32)
        } else {
            None
        }
    };
    if let Some(alive) = added_alive {
        param("alive_wisps", alive);
    }
    // The original immediately marks each arrival as saved, then removes it
    // after the short death animation. That both gives the sprite a dissolving
    // lifecycle and prevents 500 lingering units after the ending.
    mark_wisp_dying_with_delay(global, id, true, ENDING_WISP_DESTROY_DELAY);
    global
        .endgame
        .access_mut()
        .ending_wisps
        .push((id, position));
}

/// Keep the tracking list in sync with the engine-owned movement. The Lua
/// version gives each arrival a normal MOVE order and lets the engine advance
/// it; manually setting positions here changed its speed and pathing.
pub fn move_ending_wisps(global: &Global) {
    let wisps = global.endgame.access().ending_wisps.clone();
    let mut still_alive = Vec::new();
    for (id, target) in wisps {
        if spring::get_unit_position(id, false, false).is_err() {
            continue;
        }
        still_alive.push((id, target));
    }
    global.endgame.access_mut().ending_wisps = still_alive;
}

pub fn mark_wisp_dying(global: &Global, id: i32, saved: bool) {
    mark_wisp_dying_with_delay(global, id, saved, WISP_DESTROY_DELAY);
}

fn mark_wisp_dying_with_delay(global: &Global, id: i32, saved: bool, delay: i32) {
    if !global.world.access().npc_wisps.contains(&id) {
        return;
    }
    let frame = global.clock.access().frame;
    let destroy_frame = frame + delay;
    {
        let mut endgame = global.endgame.access_mut();
        if endgame
            .dying_wisps
            .iter()
            .any(|(candidate, _, _)| *candidate == id)
        {
            return;
        }
        endgame.dying_wisps.push((id, saved, destroy_frame));
    }
    let _ = spring::set_unit_rules_param_f32(id, "is_dying", 1.0, PUBLIC);
    let _ = spring::set_unit_rules_param_f32(
        id,
        "is_being_saved",
        if saved { 1.0 } else { 0.0 },
        PUBLIC,
    );
    let _ = spring::set_unit_rules_param_f32(id, "killed_frame", frame as f32, PUBLIC);
    let _ = spring::set_unit_rules_param_f32(id, "destroy_frame", destroy_frame as f32, PUBLIC);
    // unit_wisp_death.lua used to own this lifecycle.  Preserve its audible
    // feedback now that the timing and bookkeeping are implemented here.
    if saved {
        let volume = if global.endgame.access().game_over {
            3.0
        } else {
            100.0
        };
        play_unit_sound("sounds/freed.ogg", volume, id);
    } else {
        play_unit_sound("sounds/death.ogg", 100.0, id);
    }
}

pub fn expire_dying_wisps(global: &Global) -> Vec<(i32, bool, i32)> {
    let now = global.clock.access().frame;
    let mut expired = Vec::new();
    let mut endgame = global.endgame.access_mut();
    endgame.dying_wisps.retain(|(id, saved, frame)| {
        if *frame > now {
            return true;
        }
        expired.push((*id, *saved));
        false
    });
    for &(_id, saved) in &expired {
        if saved {
            endgame.saved_wisps += 1;
            param("saved_wisps", endgame.saved_wisps as f32);
        } else {
            endgame.killed_wisps += 1;
            param("killed_wisps", endgame.killed_wisps as f32);
        }
    }
    drop(endgame);
    // A normal engine destruction waits for the legacy wisp script's death
    // completion.  That script is intentionally not executed by the Core
    // port, so remove the controller state and model immediately just as the
    // original completed callback would have done.
    let mut world = global.world.access_mut();
    world
        .npc_wisps
        .retain(|candidate| !expired.iter().any(|(id, _)| id == candidate));
    param("alive_wisps", world.npc_wisps.len() as f32);
    expired
        .into_iter()
        .map(|(id, saved)| (id, saved, -1))
        .collect()
}
