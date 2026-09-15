use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::triggers;

#[derive(Default)]
pub struct TriggersGadget;

impl TriggersGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for TriggersGadget {
    fn name(&self) -> &'static str {
        "Triggers"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame % 5 == 0 {
            triggers::update_triggers(ctx.global());
        }
    }
}
