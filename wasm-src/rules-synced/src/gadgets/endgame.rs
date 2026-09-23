use spring_addons::rules::Gadget;

use crate::endgame;
use crate::state::{Global, param};

// game_end.lua declares SPAWN_FRAME_CD = 10, but never updates spawnedTime.
// Once the ending starts, frame - 0 is already greater than 10, so the actual
// Lua behavior is one arrival per GameFrame.
const ENDING_SPAWN_INTERVAL: i32 = 0;

/// End-of-game sequencing, ported from `game_end.lua`.
#[derive(Default)]
pub struct EndgameGadget;

impl EndgameGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for EndgameGadget {
    fn name(&self) -> &'static str {
        "Endgame"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        param("game_over", 0.0);
        param("saved_wisps", 0.0);
        param("killed_wisps", 0.0);
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        let expired = endgame::expire_dying_wisps(global);
        global.pending.access_mut().destroys.extend(expired);
        endgame::update_game_end(global);

        let wisp_id = global.wisp.access().id;
        endgame::move_ending_wisps(global);
        let sequence_done = {
            let (release_wisps, end_spawned, end_spawn_frame, sequence_complete) = {
                let endgame = global.endgame.access();
                (
                    endgame.release_wisps,
                    endgame.end_spawned,
                    endgame.end_spawn_frame,
                    endgame.sequence_complete,
                )
            };
            if !release_wisps || wisp_id < 0 || sequence_complete {
                false
            } else {
                if end_spawned < endgame::ENDING_SPAWN_AMOUNT
                    && frame - end_spawn_frame > ENDING_SPAWN_INTERVAL
                {
                    let mut endgame = global.endgame.access_mut();
                    endgame.end_spawned += 1;
                    endgame.end_spawn_frame = frame;
                    global.pending.access_mut().spawn_end_wisp = true;
                    false
                } else if end_spawned >= endgame::ENDING_SPAWN_AMOUNT {
                    let mut endgame = global.endgame.access_mut();
                    endgame.sequence_complete = true;
                    true
                } else {
                    false
                }
            }
        };
        if sequence_done {
            param("game_over_sequence", 1.0);
        }
    }
}
