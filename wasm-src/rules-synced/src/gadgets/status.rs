use core::cell::Cell;

use partsneeded_shared::Status;
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Sends the UI the player's and the ending's status whenever it changes.
#[derive(Default)]
pub struct StatusGadget {
    last_sent: Cell<Option<Status>>,
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
        if self.last_sent.replace(Some(status)) != Some(status) {
            let _ = spring::send_to_unsynced(&status.encode());
        }
    }
}
