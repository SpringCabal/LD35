use core::cell::Cell;

use partsneeded_shared::{BodyPart, BodyParts};
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Heartbeat interval, so the UI still resyncs if a message is ever missed.
const HEARTBEAT_FRAMES: i32 = 15;

/// What the UI is told about the player and the ending.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Status {
    spirit: bool,
    parts_left: usize,
    game_over: bool,
    wisp: i32,
    parts: BodyParts,
    ending_type: i32,
    sequence_complete: bool,
}

/// Sends the status to the UI whenever it changes, and on a heartbeat.
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

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let status = {
            let wisp = ctx.global().wisp.access();
            let endgame = ctx.global().endgame.access();
            Status {
                spirit: wisp.spirit,
                parts_left: wisp.body_parts.len(),
                game_over: endgame.game_over,
                wisp: wisp.id,
                parts: wisp.collected_parts,
                ending_type: endgame.ending_type,
                sequence_complete: endgame.sequence_complete,
            }
        };
        // Send on change, not only on the heartbeat: the UI's lever-click
        // guard must see a form change at once.
        if self.last_sent.get() == Some(status) && frame % HEARTBEAT_FRAMES != 0 {
            return;
        }
        self.last_sent.set(Some(status));
        let payload = alloc::format!(
            "PARTS_STATUS|{}|{}|{}|{}|{}|{}|{}|{}",
            i32::from(status.spirit),
            i32::from(status.parts.contains(BodyPart::Eyes)),
            status.parts_left,
            i32::from(status.game_over),
            status.wisp,
            status.parts.bits(),
            status.ending_type,
            i32::from(status.sequence_complete),
        );
        let _ = spring::send_to_unsynced(&payload);
    }
}
