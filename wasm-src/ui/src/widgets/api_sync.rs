use partsneeded_shared::{Command, Status};
use spring_addons::ui::Widget;

use crate::state::Global;

const PENDING_TIMEOUT: i32 = 30;

#[derive(Default)]
pub struct ApiSyncWidget;

impl ApiSyncWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for ApiSyncWidget {
    fn name(&self) -> &'static str {
        "ApiSync"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        crate::commands::send(spring, Command::Hello);
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        let mut player = ctx.global().player.access_mut();
        if player.pending_spirit.is_none()
            || spring.game().current_frame().unwrap_or(0) - player.pending_since <= PENDING_TIMEOUT
        {
            return;
        }
        player.pending_spirit = None;
        let (predicted, confirmed) = (player.spirit, player.rules_spirit);
        if predicted == confirmed {
            return;
        }
        player.form.begin(spring, predicted, confirmed);
        player.spirit = confirmed;
        let wisp = player.wisp;
        drop(player);
        crate::show_body(spring, wisp, !confirmed);
    }

    fn recv_from_synced(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        message: &[u8],
    ) {
        let spring = ctx.spring();
        let Some(status) = core::str::from_utf8(message).ok().and_then(Status::decode) else {
            return;
        };
        let new_spirit = status.spirit;
        let mut player = ctx.global().player.access_mut();
        player.rules_spirit = new_spirit;

        let predicted = player.pending_spirit.is_some();
        let stale = match player.pending_spirit {
            Some(wanted) if wanted == new_spirit => {
                player.pending_spirit = None;
                false
            }
            Some(_) => true,
            None => false,
        };

        let was_spirit = player.spirit;
        if !stale {
            player.form.begin(spring, was_spirit, new_spirit);
            player.spirit = new_spirit;
        }
        player.game_over = status.game_over;
        player.wisp = status.wisp;
        player.parts = status.parts;
        player.ending = status.ending;
        if status.sequence_complete != player.game_over_sequence {
            player.sequence_since = if status.sequence_complete {
                spring.game().current_frame().unwrap_or(0)
            } else {
                -1
            };
        }
        player.game_over_sequence = status.sequence_complete;
        drop(player);
        if stale {
            return;
        }
        let wisp = status.wisp;
        crate::show_body(spring, wisp, !new_spirit);
        if was_spirit != new_spirit && !predicted {
            let position = spring
                .units_info()
                .get_unit_position(wisp, Default::default())
                .unwrap_or_else(|_| spring::Float3::default());
            let _ = spring.sound().play_sound_file(
                "sounds/changestate.ogg",
                1.0,
                position,
                spring::Float3::default(),
                0,
            );
        }
    }
}
