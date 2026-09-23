use spring_addons::ui::Widget;

use crate::state::{GAME_END_DURATION, Global};

pub struct GuiHudWidget {
    sequence_start: core::cell::Cell<i32>,
}

impl Default for GuiHudWidget {
    fn default() -> Self {
        Self {
            sequence_start: core::cell::Cell::new(-1),
        }
    }
}

impl GuiHudWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiHudWidget {
    fn name(&self) -> &'static str {
        "GuiHud"
    }

    fn draw_screen(&self, ctx: &spring_addons::AddonContext<'_, Global>, width: i32, height: i32) {
        let global = ctx.global().player.access();
        if !global.game_over && global.spirit && global.eyes && !global.space_used {
            let _ = spring::gfx::begin_text(false);
            let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
            let _ = spring::gfx::text(
                "Press Space",
                width as f32 * 0.4,
                height as f32 * 0.4,
                20.0,
                "o",
            );
            let _ = spring::gfx::end_text(0);
        }
        if !global.game_over_sequence {
            self.sequence_start.set(-1);
            return;
        }
        let frame = crate::state::current_frame();
        let start = match self.sequence_start.get() {
            start if start >= 0 => start,
            _ => {
                self.sequence_start.set(frame);
                frame
            }
        };
        // gfx_eyes_effect.lua waits for the fade and bright spirit hold before
        // calling ShowEndgameUI. Keep that one-shot timing here, without a
        // Chili window that could be accidentally recreated every frame.
        if frame - start <= GAME_END_DURATION * 2 {
            return;
        }
        let status = match global.ending_type {
            0 => "Welcome back, Master.   Ending #1",
            1 => "The souls have been freed.   Ending #2",
            2 => "Their sacrifice is accepted.   Ending #3",
            _ => "You have achieved nothing noteworthy.   Ending #4",
        };
        let _ = spring::gfx::begin_text(false);
        let _ = spring::gfx::color(0.8, 0.9, 1.0, 1.0);
        let _ = spring::gfx::text(
            status,
            width as f32 / 2.0,
            height as f32 / 2.0,
            16.0,
            "cson",
        );
        let _ = spring::gfx::text(
            "Click or press R to restart     Ctrl-Q: return to menu",
            width as f32 / 2.0,
            height as f32 / 2.0 - 32.0,
            12.0,
            "cson",
        );
        let _ = spring::gfx::end_text(0);
    }
}
