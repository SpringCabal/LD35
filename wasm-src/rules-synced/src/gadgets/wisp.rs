use spring_addons::rules::Gadget;

use crate::state::{Global, param};
use crate::wisp;

/// Frames a movement message stays valid.  The UI resends it every frame
/// while a key is held; this bounds how long a lost release keeps the wisp
/// moving.
const MOVE_STALE_FRAMES: i32 = 6;

/// The player wisp controller, ported from `unit_wisp_controller.lua`.
#[derive(Default)]
pub struct WispGadget;

impl WispGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for WispGadget {
    fn name(&self) -> &'static str {
        "Wisp"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        param("spiritMode", 1.0);
    }

    /// The callouts made while the wisp is borrowed (positions, move goal,
    /// velocity, rotation, rules params) raise no callins; the orders and
    /// piece changes go through the reaper.
    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        let mut wisp = global.wisp.access_mut();
        if wisp.id >= 0 {
            wisp::cancel_upward_velocity(&wisp);
            #[cfg(feature = "headless-integration")]
            {
                let move_goal_accepted = wisp::move_wisp(&wisp);
                if wisp.movement.is_some() && !move_goal_accepted {
                    let _ = spring::echo("[PartsNeeded-Rust-Test] move goal was rejected", "");
                }
            }
            #[cfg(not(feature = "headless-integration"))]
            {
                let _ = wisp::move_wisp(&wisp);
            }
            let collected = wisp::collect_body_parts(&mut wisp);
            global.pending.access_mut().destroys.extend(collected);
        }
        if wisp.movement.is_some() && frame > wisp.movement_frame + MOVE_STALE_FRAMES {
            wisp.movement = None;
        }
        wisp::rotate_body_parts(&wisp);
        let stop = wisp.id >= 0 && wisp.movement.is_none() && !wisp.stop_issued;
        wisp.stop_issued |= stop;
        let pieces = core::mem::take(&mut wisp.pieces_dirty).then_some((
            wisp.id,
            wisp.spirit,
            wisp.collected_parts,
        ));
        let id = wisp.id;
        drop(wisp);

        let mut pending = global.pending.access_mut();
        if stop {
            pending.stop_wisp = Some(id);
        }
        if pieces.is_some() {
            pending.wisp_pieces = pieces;
        }
    }
}
