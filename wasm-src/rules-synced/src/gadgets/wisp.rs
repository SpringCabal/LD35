pub mod behavior;
use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::log::SECTION;

use crate::state::Global;

#[derive(Default)]
pub struct WispGadget {
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
        let spring = ctx.spring();
        let mut wisp = ctx.global().wisp.access_mut();
        if wisp.id >= 0 {
            behavior::cancel_upward_velocity(spring, &wisp);
            let rejected =
                wisp.steering.direction().is_some() && !behavior::move_wisp(spring, &wisp);
            if rejected && !self.move_rejected.get() {
                spring::log_warning!(spring, SECTION, "the engine refused the wisp's move goal");
            }
            self.move_rejected.set(rejected);
            crate::units::destroy(spring, behavior::collect_body_parts(spring, &mut wisp));
        }
        behavior::rotate_body_parts(spring, &wisp);
        if core::mem::take(&mut wisp.pieces_dirty) {
            behavior::sync_wisp_pieces(spring, wisp.id, wisp.spirit, wisp.collected_parts);
        }
        if wisp.id >= 0 && wisp.steering.take_stop() {
            behavior::stop_wisp(spring, wisp.id);
        }
    }
}
