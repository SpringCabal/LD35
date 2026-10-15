pub mod behavior;
use spring::commands;
use spring_addons::rules::Gadget;

use crate::state::Global;

const ENDING_SPAWN_INTERVAL: i32 = 0;

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
        crate::units::destroy(behavior::expire_dying_wisps(global));
        behavior::update_game_end(global);

        let wisp_id = global.wisp.access().id;
        behavior::move_ending_wisps(global);
        let mut endgame = global.endgame.access_mut();
        if !endgame.release_wisps || wisp_id < 0 || endgame.sequence_complete {
            return;
        }
        if endgame.end_spawned >= behavior::ENDING_SPAWN_AMOUNT {
            endgame.sequence_complete = true;
        } else if frame - endgame.end_spawn_frame > ENDING_SPAWN_INTERVAL {
            endgame.end_spawned += 1;
            endgame.end_spawn_frame = frame;
            commands::run(|| crate::with_global(behavior::spawn_ending_wisp));
        }
    }
}
