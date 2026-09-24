use core::cell::RefCell;

use spring_addons::ui::Widget;

use crate::audio::{self, Audio};
use crate::state::Global;

#[derive(Default)]
pub struct SndAmbienceWidget {
    audio: RefCell<Audio>,
}

impl SndAmbienceWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for SndAmbienceWidget {
    fn name(&self) -> &'static str {
        "SndAmbience"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let (parts, spirit, wisp) = {
            let player = ctx.global().player.access();
            (player.parts, player.spirit, player.wisp)
        };
        audio::play_ambient(&mut self.audio.borrow_mut(), parts, spirit, wisp);
    }
}
