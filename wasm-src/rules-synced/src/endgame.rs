//! End-of-game sequencing and NPC wisp fates.

use alloc::vec::Vec;

use partsneeded_shared::Ending;
use spring::commands;

use crate::sound::play_unit_sound;
use crate::state::{ENDING_WISP_DESTROY_DELAY, Global, PUBLIC, WISP_DESTROY_DELAY};

pub fn update_game_end(global: &Global) {
    let (has_all_parts, already_over, alive_wisps, saved_wisps, killed_wisps) = {
        let wisp = global.wisp.access();
        let endgame = global.endgame.access();
        let world = global.world.access();
        (
            wisp.collected_parts.is_complete(),
            endgame.game_over,
            world.npc_wisps.len(),
            endgame.saved_wisps,
            endgame.killed_wisps,
        )
    };
    if !has_all_parts || already_over {
        return;
    }

    // As in game_end.lua, the counters decide as they are now: wisps still
    // being freed or killed count as alive.
    let ending = Ending::from_counts(alive_wisps, saved_wisps, killed_wisps);
    let souls_freed = ending == Ending::SoulsFreed;
    {
        let mut endgame = global.endgame.access_mut();
        endgame.game_over = true;
        endgame.ending = Some(ending);
        // Only the souls-freed ending has a delayed visual sequence.  The
        // other three outcomes go straight to the end screen after the
        // screen-effect fade, so expose that sequence as complete immediately
        // through the same status path used by the WASM UI.
        endgame.sequence_complete = !souls_freed;
        endgame.release_wisps = souls_freed;
    }
    // Input is rejected from now on, so the release of a held movement key
    // would never arrive: stop the wisp here.  (unit_wisp_controller.lua's
    // last movement message goes stale six frames later.)
    global.wisp.access_mut().steering.release();
    if souls_freed {
        let wisp_id = {
            let mut wisp = global.wisp.access_mut();
            wisp.spirit = true;
            wisp.steering.release_quietly();
            wisp.pieces_dirty = true;
            wisp.id
        };
        // Match game_end.lua: freeze the player at the ceremony while the
        // incoming spirits converge on it. Input is rejected once game_over
        // is set, but an already-issued move goal would otherwise keep it
        // drifting during the ending.
        crate::log::check(
            "freezing the player for the ending",
            wisp_id,
            spring::move_ctrl::move_ctrl(wisp_id, true),
        );
    }
}

/// Lua's game_end.lua emits 500 short-lived saved wisps. Although it declares
/// a ten-frame cooldown, its `spawnedTime` local is never updated, so the
/// effective behavior is one arrival per GameFrame.
pub const ENDING_SPAWN_AMOUNT: i32 = 500;

/// How far from the player the ending's arrivals appear, per axis: game_end.lua
/// samples [-300, 300] and pushes the sample another 100 elmos away from zero.
const ENDING_SPAWN_DISTANCE: f32 = 300.0;
const ENDING_SPAWN_DISTANCE_MIN: f32 = 100.0;

/// One axis of game_end.lua's spawn offset, from the synced engine RNG that
/// its `math.random` draws from.
fn ending_offset() -> f32 {
    let offset = (0.5 - spring::random::synced::float()) * ENDING_SPAWN_DISTANCE * 2.0;
    if offset > 0.0 {
        offset + ENDING_SPAWN_DISTANCE_MIN
    } else {
        offset - ENDING_SPAWN_DISTANCE_MIN
    }
}

pub fn spawn_ending_wisp(global: &Global) {
    let wisp_id = global.wisp.access().id;
    let Some(position) = spring::get_unit_position(wisp_id, false, false).ok() else {
        return;
    };
    let dx = ending_offset();
    let dz = ending_offset();
    let spawn_position = spring::Float3::new(position.x + dx, position.y, position.z + dz);
    crate::level::spawn_unit(
        "npcwisp",
        crate::defs::defs().npcwisp,
        spawn_position,
        move |id| {
            // Match game_end.lua: a normal MOVE order, so the engine's own
            // movement (acceleration, speed) carries each arrival to the player.
            commands::give_order_to_unit(
                id,
                spring::cmd::MOVE,
                &[position.x, position.y, position.z],
                0,
                0,
            )
            .then(move |result| {
                crate::log::check("ending wisp MOVE order", id, result);
            });
            // The original immediately marks each arrival as saved, then
            // removes it after the short death animation. That both gives the
            // sprite a dissolving lifecycle and prevents 500 lingering units
            // after the ending. Roster's UnitCreated has listed it as an NPC
            // wisp by now.
            crate::with_global(|global| {
                mark_wisp_dying_with_delay(global, id, true, ENDING_WISP_DESTROY_DELAY);
                global
                    .endgame
                    .access_mut()
                    .ending_wisps
                    .push((id, position));
            });
        },
    );
}

/// Drop ending wisps that no longer exist from the tracking list.
pub fn move_ending_wisps(global: &Global) {
    global
        .endgame
        .access_mut()
        .ending_wisps
        .retain(|(id, _)| spring::get_unit_position(*id, false, false).is_ok());
}

pub fn mark_wisp_dying(global: &Global, id: i32, saved: bool) {
    mark_wisp_dying_with_delay(global, id, saved, WISP_DESTROY_DELAY);
}

fn mark_wisp_dying_with_delay(global: &Global, id: i32, saved: bool, delay: i32) {
    if !global.world.access().npc_wisps.contains(&id) {
        return;
    }
    let frame = spring::game::current_frame();
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
    // The UI draws the death or rescue from these.
    for (param, value) in [
        ("is_dying", 1.0),
        ("killed_frame", frame as f32),
        ("destroy_frame", destroy_frame as f32),
    ] {
        crate::log::check(
            param,
            id,
            spring::set_unit_rules_param_f32(id, param, value, PUBLIC),
        );
    }
    // unit_wisp_death.lua's audible feedback.
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
    let now = spring::game::current_frame();
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
        } else {
            endgame.killed_wisps += 1;
        }
    }
    drop(endgame);
    // Leave the alive set now, as the original death callback did, rather
    // than when the queued destruction reaches UnitDestroyed.
    let mut world = global.world.access_mut();
    world
        .npc_wisps
        .retain(|candidate| !expired.iter().any(|(id, _)| id == candidate));
    expired
        .into_iter()
        .map(|(id, saved)| (id, saved, -1))
        .collect()
}
