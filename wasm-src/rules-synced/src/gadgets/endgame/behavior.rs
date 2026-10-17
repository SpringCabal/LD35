use alloc::vec::Vec;

use partsneeded_shared::Ending;

use crate::sound::play_unit_sound;
use crate::state::{ENDING_WISP_DESTROY_DELAY, Global, PUBLIC, WISP_DESTROY_DELAY};

pub fn update_game_end(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
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

    let ending = Ending::from_counts(alive_wisps, saved_wisps, killed_wisps);
    let souls_freed = ending == Ending::SoulsFreed;
    {
        let mut endgame = global.endgame.access_mut();
        endgame.game_over = true;
        endgame.ending = Some(ending);
        endgame.sequence_complete = !souls_freed;
        endgame.release_wisps = souls_freed;
    }
    global.wisp.access_mut().steering.release();
    if souls_freed {
        let wisp_id = {
            let mut wisp = global.wisp.access_mut();
            wisp.spirit = true;
            wisp.steering.release_quietly();
            wisp.pieces_dirty = true;
            wisp.id
        };
        crate::log::check(
            spring,
            "freezing the player for the ending",
            wisp_id,
            spring.move_ctrl().move_ctrl(wisp_id, true),
        );
    }
}

pub const ENDING_SPAWN_AMOUNT: i32 = 500;

const ENDING_SPAWN_DISTANCE: f32 = 300.0;
const ENDING_SPAWN_DISTANCE_MIN: f32 = 100.0;

fn ending_offset(spring: &spring::Spring<'_, spring::RulesSynced>) -> f32 {
    let offset = (0.5
        - spring
            .synced_random()
            .next_float()
            .expect("synced random float"))
        * ENDING_SPAWN_DISTANCE
        * 2.0;
    if offset > 0.0 {
        offset + ENDING_SPAWN_DISTANCE_MIN
    } else {
        offset - ENDING_SPAWN_DISTANCE_MIN
    }
}

pub fn spawn_ending_wisp(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let wisp_id = global.wisp.access().id;
    let Some(position) = spring
        .units_info()
        .get_unit_position(wisp_id, Default::default())
        .ok()
    else {
        return;
    };
    let dx = ending_offset(spring);
    let dz = ending_offset(spring);
    let spawn_position = spring::Float3::new(position.x + dx, position.y, position.z + dz);
    crate::level::spawn_unit(
        spring,
        "npcwisp",
        crate::defs::defs(spring).npcwisp,
        spawn_position,
        move |spring, id| {
            crate::with_addon_context(|ctx| {
                ctx.commands().give_order_to_unit(
                    id,
                    spring::cmd::MOVE,
                    &[position.x, position.y, position.z],
                    0,
                    0,
                )
            })
            .then(move |spring, result| {
                crate::log::check(spring, "ending wisp MOVE order", id, result);
            });
            crate::with_global(|global| {
                mark_wisp_dying_with_delay(spring, global, id, true, ENDING_WISP_DESTROY_DELAY);
                global
                    .endgame
                    .access_mut()
                    .ending_wisps
                    .push((id, position));
            });
        },
    );
}

pub fn move_ending_wisps(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    global.endgame.access_mut().ending_wisps.retain(|(id, _)| {
        spring
            .units_info()
            .get_unit_position(*id, Default::default())
            .is_ok()
    });
}

pub fn mark_wisp_dying(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    id: i32,
    saved: bool,
) {
    mark_wisp_dying_with_delay(spring, global, id, saved, WISP_DESTROY_DELAY);
}

fn mark_wisp_dying_with_delay(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    id: i32,
    saved: bool,
    delay: i32,
) {
    if !global.world.access().npc_wisps.contains(&id) {
        return;
    }
    let frame = spring.game().current_frame().unwrap_or(0);
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
    for (param, value) in [
        ("is_dying", 1.0),
        ("killed_frame", frame as f32),
        ("destroy_frame", destroy_frame as f32),
    ] {
        crate::log::check(
            spring,
            param,
            id,
            spring
                .rules_params()
                .set_unit_rules_param_f32(id, param, value, PUBLIC),
        );
    }
    if saved {
        let volume = if global.endgame.access().game_over {
            3.0
        } else {
            100.0
        };
        play_unit_sound(spring, "sounds/freed.ogg", volume, id);
    } else {
        play_unit_sound(spring, "sounds/death.ogg", 100.0, id);
    }
}

pub fn expire_dying_wisps(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
) -> Vec<(i32, bool, i32)> {
    let now = spring.game().current_frame().unwrap_or(0);
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
    let mut world = global.world.access_mut();
    world
        .npc_wisps
        .retain(|candidate| !expired.iter().any(|(id, _)| id == candidate));
    expired
        .into_iter()
        .map(|(id, saved)| (id, saved, -1))
        .collect()
}
