use alloc::string::String;
use core::cell::RefCell;

use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::wisp;

/// Heartbeat interval, so the UI still resyncs if a message is ever missed.
const HEARTBEAT_FRAMES: i32 = 15;

#[derive(Default)]
pub struct StatusGadget {
    last_sent: RefCell<String>,
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
        let wisp = ctx.global().wisp.access();
        let endgame = ctx.global().endgame.access();
        let payload = alloc::format!(
            "PARTS_STATUS|{}|{}|{}|{}|{}|{}|{}|{}",
            i32::from(wisp.spirit),
            i32::from(wisp.has_eyes),
            wisp.body_parts.len(),
            i32::from(endgame.game_over),
            wisp.id,
            wisp::body_part_mask(&wisp),
            endgame.ending_type,
            i32::from(endgame.sequence_complete),
        );

        // Send on change, not just on the heartbeat.  At one message every
        // fifteen frames the UI's copy of spiritMode/body_mask could be half a
        // second out of date, which was long enough for the lever-click guard
        // to refuse a legitimate click right after taking body form.
        let mut last = self.last_sent.borrow_mut();
        if *last != payload || frame % HEARTBEAT_FRAMES == 0 {
            let _ = spring::send_to_unsynced(&payload);
            last.clear();
            last.push_str(&payload);
        }
    }
}
