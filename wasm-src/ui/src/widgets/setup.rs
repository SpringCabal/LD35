use spring_addons::register_widgets;
use spring_addons::ui::WidgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            ApiSyncWidget::new(),
            HideDefaultLayoutWidget::new(),
            GuiWispControlWidget::new(),
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
