//! Gallery mode (`springcabal::gallery`): only the widgets that draw what a
//! player sees in the world.

use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::gallery::Gallery;

use super::*;
use crate::state::Global;

/// The world effects: floor decals, the wisp sprites and the screen pass.
/// The lever highlight follows the cursor, so it stays out.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    Gallery::new(handler)
        .widget(ApiSyncWidget::new())
        .widget(HideDefaultInterface::new(setup::HIDDEN_INTERFACE))
        .widget(GfxBloodWidget::new())
        .widget(GfxEyesEffectWidget::new())
        .announce("partsneeded");
}
