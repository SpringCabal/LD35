use alloc::rc::Rc;

use partsneeded_shared::{Command, Status};

use spring_addons::rules::Gadget;
use springcabal::message::SendOnChange;

use crate::state::Global;

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
        let _spring = ctx.spring();
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
        self.sent.send_if_changed(status, |status| {
            let message = status.encode();
            let sent = Rc::clone(&self.sent);
            crate::with_addon_context(|ctx| {
                ctx.commands()
                    .queue(move |spring| springcabal::message::send_to_ui(spring, &message))
            })
            .then(move |_spring, delivered| {
                if !delivered {
                    sent.reset();
                }
            });
            true
        });
    }
}
