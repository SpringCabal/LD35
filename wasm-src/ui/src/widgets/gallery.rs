//! Gallery mode, for the verification galleries
//! (`verification/fixtures/gallery/gallery.lua` in springcabal-games).  They
//! set the `gallery` modoption, place the camera themselves and capture single
//! units, so only the widgets that draw what a player sees in the world stay:
//! no HUD, input, camera or sound.

use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};

use super::*;
use crate::state::Global;

/// Whether the `gallery` modoption is set.
pub fn active() -> bool {
    spring::game_options::get_mod_option("gallery")
        .ok()
        .flatten()
        .is_some_and(|mode| !mode.is_empty())
}

/// The world effects: floor decals, the wisp sprites and the screen pass.
/// The lever highlight follows the cursor, so it stays out.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            ApiSyncWidget::new(),
            HideDefaultInterface::new(setup::HIDDEN_INTERFACE),
            GfxBloodWidget::new(),
            GfxEyesEffectWidget::new(),
        ]
    );
    spring::log_notice!(
        "partsneeded",
        "gallery mode: ApiSync HideDefaultInterface GfxBlood GfxEyesEffect"
    );
}
