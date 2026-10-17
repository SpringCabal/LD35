use partsneeded_shared::BodyPart;
use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct GuiHudWidget;

impl GuiHudWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GuiHudWidget {
    fn name(&self) -> &'static str {
        "GuiHud"
    }

    fn draw_screen(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        let spring = ctx.spring();
        let global = ctx.global().player.access();
        if !global.game_over
            && global.spirit
            && global.parts.contains(BodyPart::Eyes)
            && !global.space_used
        {
            let _ = spring.gfx().begin_text(false);
            let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
            let _ = spring.gfx().text(
                "Press Space",
                width as f32 * 0.4,
                height as f32 * 0.4,
                20.0,
                "o",
            );
            let _ = spring.gfx().end_text();
        }
        if !global.end_screen_shown(spring) {
            return;
        }
        let Some(ending) = global.ending else {
            return;
        };
        let status = ending.text();
        let _ = spring.gfx().begin_text(false);
        let _ = spring.gfx().color(0.8, 0.9, 1.0, 1.0);
        let _ = spring.gfx().text(
            status,
            width as f32 / 2.0,
            height as f32 / 2.0,
            16.0,
            "cson",
        );
        let _ = spring.gfx().text(
            "Click or press R to restart     Ctrl-Q: return to menu",
            width as f32 / 2.0,
            height as f32 / 2.0 - 32.0,
            12.0,
            "cson",
        );
        let _ = spring.gfx().end_text();
    }
}
