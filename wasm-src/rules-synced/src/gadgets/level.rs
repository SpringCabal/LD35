use core::cell::Cell;

use spring_addons::rules::Gadget;

use crate::state::Global;

/// Frame the level is spawned on, after the map's own units exist.
const SPAWN_FRAME: i32 = 3;

/// Spawns the level once, from game_custom_spawn.lua's tables.
#[derive(Default)]
pub struct LevelGadget {
    requested: Cell<bool>,
}

impl LevelGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for LevelGadget {
    fn name(&self) -> &'static str {
        "Level"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame >= SPAWN_FRAME && !self.requested.replace(true) {
            ctx.global().pending.access_mut().spawn_level = true;
        }
    }
}
