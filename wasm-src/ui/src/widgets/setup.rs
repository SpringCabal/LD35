use spring::default_interface::{
    CLOCK, CONSOLE, MINIMAP, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::HideDefaultInterface;
use spring_addons::register_widgets;
use spring_addons::ui::WidgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            ApiSyncWidget::new(),
            // hide_default_layout.lua's list, plus the distance icons.
            HideDefaultInterface::new(
                CONSOLE | RESOURCE_BAR | TOOLTIP | CLOCK | PLAYER_INFO | MINIMAP | UNIT_ICONS
            ),
            CameraFollowWidget::new(),
            GuiWispControlWidget::new(),
            SndMusicWidget::new(),
            SndAmbienceWidget::new(),
            GfxBloodWidget::new(),
            GfxEyesEffectWidget::new(),
            GfxHighlightWidget::new(),
            GuiHudWidget::new(),
        ]
    );
    #[cfg(feature = "unit-script-probe")]
    register_widgets!(handler, [UnitScriptProbeWidget::new()]);
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgE2eWidget::new()]);
}
