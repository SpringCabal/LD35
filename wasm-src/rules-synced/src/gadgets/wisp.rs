use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::log::SECTION;

use crate::state::{Global, param};
use crate::wisp;

/// Frames a movement message stays valid.  The UI resends it every frame
/// while a key is held; this bounds how long a lost release keeps the wisp
/// moving.
const MOVE_STALE_FRAMES: i32 = 6;

/// The player wisp controller, ported from `unit_wisp_controller.lua`.
#[derive(Default)]
pub struct WispGadget {
    /// Whether the engine refused the last move goal; logged when it starts.
    move_rejected: Cell<bool>,
}

impl WispGadget {
    pub fn new() -> Self {
        Self::default()
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
    /// velocity, rotation, rules params, piece visibility) raise no callins;
    /// the STOP order and the pickups' removal run after the frame.
    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        let mut wisp = global.wisp.access_mut();
        if wisp.id >= 0 {
            wisp::cancel_upward_velocity(&wisp);
            let rejected = wisp.movement.is_some() && !wisp::move_wisp(&wisp);
            if rejected && !self.move_rejected.get() {
                spring::log_warning!(SECTION, "the engine refused the wisp's move goal");
            }
            self.move_rejected.set(rejected);
            let collected = wisp::collect_body_parts(&mut wisp);
            crate::units::destroy_later(ctx, collected);
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

        if let Some((id, spirit, parts)) = pieces {
            wisp::sync_wisp_pieces(id, spirit, parts);
        }
        if stop {
            // An order can raise command callins synchronously.
            ctx.delay(move |_| {
                crate::log::check(
                    "STOP order",
                    id,
                    spring::give_order_to_unit(id, spring::cmd::STOP, &[], 0, 0),
                );
            });
        }
    }
}
