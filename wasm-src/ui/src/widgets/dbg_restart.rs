use core::cell::Cell;

use partsneeded_shared::{Command, Ending};
use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct DbgRestartWidget {
    sent: Cell<bool>,
}

impl DbgRestartWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for DbgRestartWidget {
    fn name(&self) -> &'static str {
        "DbgRestart"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        let end_screen = {
            let player = ctx.global().player.access();
            player.ending == Some(Ending::SoulsFreed) && player.end_screen_shown(spring)
        };
        if end_screen && !self.sent.replace(true) {
            spring::log_info!(spring, "partsneeded", "restarting from the end screen");
            crate::commands::send(spring, Command::Restart);
        }
    }
}
