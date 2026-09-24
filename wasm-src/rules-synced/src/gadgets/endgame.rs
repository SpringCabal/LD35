use spring_addons::rules::Gadget;

use crate::endgame;
use crate::state::Global;

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

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        crate::units::destroy_later(ctx, endgame::expire_dying_wisps(global));
        endgame::update_game_end(global);

        let wisp_id = global.wisp.access().id;
        endgame::move_ending_wisps(global);
        let mut endgame = global.endgame.access_mut();
        if !endgame.release_wisps || wisp_id < 0 || endgame.sequence_complete {
            return;
        }
        if endgame.end_spawned >= endgame::ENDING_SPAWN_AMOUNT {
            endgame.sequence_complete = true;
        } else if frame - endgame.end_spawn_frame > ENDING_SPAWN_INTERVAL {
            endgame.end_spawned += 1;
            endgame.end_spawn_frame = frame;
            ctx.delay(endgame::spawn_ending_wisp);
        }
    }
}
