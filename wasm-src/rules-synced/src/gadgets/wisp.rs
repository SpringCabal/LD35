use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::log::SECTION;

use crate::state::Global;
use crate::wisp;

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

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        let mut wisp = ctx.global().wisp.access_mut();
        if wisp.id >= 0 {
            wisp::cancel_upward_velocity(&wisp);
            let rejected = wisp.steering.direction().is_some() && !wisp::move_wisp(&wisp);
            if rejected && !self.move_rejected.get() {
                spring::log_warning!(SECTION, "the engine refused the wisp's move goal");
            }
            self.move_rejected.set(rejected);
            crate::units::destroy(wisp::collect_body_parts(&mut wisp));
        }
        wisp::rotate_body_parts(&wisp);
        if core::mem::take(&mut wisp.pieces_dirty) {
            wisp::sync_wisp_pieces(wisp.id, wisp.spirit, wisp.collected_parts);
        }
        if wisp.id >= 0 && wisp.steering.take_stop() {
            wisp::stop_wisp(wisp.id);
        }
    }
}
