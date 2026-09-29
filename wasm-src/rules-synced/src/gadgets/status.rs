use alloc::rc::Rc;

use partsneeded_shared::{Command, Status};
use spring::commands;
use spring_addons::rules::Gadget;
use springcabal::message::SendOnChange;

use crate::state::Global;

/// Sends the UI the player's and the ending's status whenever it changes.
#[derive(Default)]
pub struct StatusGadget {
    sent: Rc<SendOnChange<Status>>,
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

    /// A UI that (re)started asks for the status; send it on the next frame
    /// even if nothing changed.  This works during the ending too, when
    /// every other command is refused.
    fn handle_lua_msg(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let hello = core::str::from_utf8(data)
            .ok()
            .and_then(Command::decode)
            .is_some_and(|command| command == Command::Hello);
        if hello {
            self.sent.reset();
        }
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
                ending: endgame.ending,
                sequence_complete: endgame.sequence_complete,
            }
        };
        // Unsent, the UI keeps showing the old form and parts: forgotten, so
        // it is retried on the next frame.
        self.sent.send_if_changed(status, |status| {
            let message = status.encode();
            let sent = Rc::clone(&self.sent);
            commands::queue(move || springcabal::message::send_to_ui(&message)).then(
                move |delivered| {
                    if !delivered {
                        sent.reset();
                    }
                },
            );
            true
        });
    }
}
