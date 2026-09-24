use partsneeded_shared::Status;
use spring_addons::rules::Gadget;
use springcabal::message::SendOnChange;

use crate::log::SECTION;
use crate::state::Global;

/// Sends the UI the player's and the ending's status whenever it changes.
#[derive(Default)]
pub struct StatusGadget {
    sent: SendOnChange<Status>,
}

impl StatusGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for StatusGadget {
    fn name(&self) -> &'static str {
        "Status"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        let status = {
            let wisp = ctx.global().wisp.access();
            let endgame = ctx.global().endgame.access();
            Status {
                spirit: wisp.spirit,
                game_over: endgame.game_over,
                wisp: wisp.id,
                parts: wisp.collected_parts,
                ending_type: endgame.ending_type,
                sequence_complete: endgame.sequence_complete,
            }
        };
        // Unsent, the UI keeps showing the old form and parts: retried on the
        // next frame.
        self.sent.send_if_changed(status, |status| {
            spring::send_to_unsynced(&status.encode())
                .inspect_err(|error| {
                    spring::log_warning!(SECTION, "sending the status failed: {error:?}")
                })
                .is_ok()
        });
    }
}
