use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::verification::WorldWidgets;

use super::*;
use crate::state::Global;

pub fn setup(handler: &mut WidgetHandler<Global>) {
    WorldWidgets::new(handler)
        .widget(ApiSyncWidget::new())
        .widget(HideDefaultInterface::new(setup::HIDDEN_INTERFACE))
        .widget(GfxBloodWidget::new())
        .widget(GfxEyesEffectWidget::new())
        .announce("partsneeded");
}
