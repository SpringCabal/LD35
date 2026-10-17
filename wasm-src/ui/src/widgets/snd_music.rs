use core::cell::Cell;

use partsneeded_shared::BodyPart;
use spring_addons::ui::Widget;
use springcabal::music::MusicLoop;

use crate::state::Global;

const FULL_MUSIC: &str = "sounds/music.ogg";
const MUFFLED_MUSIC: &str = "sounds/muffled.ogg";

pub struct SndMusicWidget {
    music: MusicLoop,
    ears: Cell<Option<bool>>,
}

impl Default for SndMusicWidget {
    fn default() -> Self {
        Self {
            music: MusicLoop::new(MUFFLED_MUSIC, 0.6),
            ears: Cell::new(None),
        }
    }
}

impl SndMusicWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for SndMusicWidget {
    fn name(&self) -> &'static str {
        "SndMusic"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        let has_ears = ctx.global().player.access().parts.contains(BodyPart::Ears);
        if self.ears.replace(Some(has_ears)) != Some(has_ears) {
            if has_ears {
                self.music.switch(FULL_MUSIC, 1.0);
            } else {
                self.music.switch(MUFFLED_MUSIC, 0.6);
            }
        }
        self.music.update(ctx, dt);
    }

    fn shutdown(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        self.music.shutdown(ctx);
    }
}
