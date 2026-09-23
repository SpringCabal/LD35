use spring_addons::ui::Widget;

use crate::audio;
use crate::state::Global;

#[derive(Default)]
pub struct SndAmbienceWidget;

impl SndAmbienceWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for SndAmbienceWidget {
    fn name(&self) -> &'static str {
        "SndAmbience"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let global = ctx.global().player.access();
        audio::update(global.parts, global.spirit, global.wisp);
    }
}
