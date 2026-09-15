use spring_addons::rules::Gadget;

use crate::state::Global;

#[derive(Default)]
pub struct LevelGadget;

impl LevelGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for LevelGadget {
    fn name(&self) -> &'static str {
        "Level"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let mut world = ctx.global().world.access_mut();
        if frame >= 3 && !world.level_requested {
            world.level_requested = true;
            ctx.global().pending.access_mut().spawn_level = true;
        }
    }
}
