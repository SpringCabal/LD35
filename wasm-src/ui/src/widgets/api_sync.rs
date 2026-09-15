use spring_addons::ui::Widget;

use crate::state::Global;

/// Frames to trust a local form prediction before assuming the rules module
/// refused the switch.  The round trip is a few frames; this is generous.
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

    fn recv_from_synced(&self, ctx: &spring_addons::AddonContext<'_, Global>, message: &[u8]) {
        let Ok(message) = core::str::from_utf8(message) else {
            return;
        };
        let mut values = message
            .strip_prefix("PARTS_STATUS|")
            .unwrap_or_default()
            .split('|')
            .filter_map(|value| value.parse::<i32>().ok());
        let parsed = (
            values.next(),
            values.next(),
            values.next(),
            values.next(),
            values.next(),
            values.next(),
            values.next(),
            values.next(),
        );
        if let (
            Some(spirit),
            Some(eyes),
            Some(remaining),
            Some(game_over),
            Some(wisp),
            Some(mask),
            Some(ending_type),
            Some(game_over_sequence),
        ) = parsed
        {
            let new_spirit = spirit != 0;
            let mut player = ctx.global().player.access_mut();

            // While a form change is in flight the snapshot still reports the
            // old form.  Keep the local prediction until the synced side agrees
            // -- or until it clearly never will, because the rules module
            // refused the switch (form-change cooldown).
            let predicted = player.pending_spirit.is_some();
            let stale = match player.pending_spirit {
                Some(wanted) if wanted == new_spirit => {
                    player.pending_spirit = None;
                    false
                }
                Some(_) => {
                    if crate::state::current_frame() - player.pending_since > PENDING_TIMEOUT {
                        player.pending_spirit = None;
                        false
                    } else {
                        true
                    }
                }
                None => false,
            };

            let was_spirit = player.spirit;
            if !stale {
                player.form.begin(was_spirit, new_spirit);
                player.spirit = new_spirit;
            }
            player.eyes = eyes != 0;
            player.remaining = remaining;
            player.game_over = game_over != 0;
            player.wisp = wisp;
            player.body_mask = mask as u8;
            player.ending_type = ending_type;
            player.game_over_sequence = game_over_sequence != 0;
            drop(player);
            if stale {
                return;
            }
            let _ = spring::unsynced_ctrl::set_unit_no_draw(wisp, spirit != 0);
            // Form changes we predicted already played this on the keypress;
            // this covers changes that originate synced-side (game over forcing
            // spirit form, for instance).
            if was_spirit != new_spirit && !predicted {
                let position = spring::get_unit_position(wisp, false, false)
                    .unwrap_or_else(|_| spring::types::Float3::default());
                let _ = spring::sound::play_sound_file(
                    "sounds/changestate.ogg",
                    1.0,
                    position,
                    spring::types::Float3::default(),
                    0,
                );
            }
        }
    }
}
