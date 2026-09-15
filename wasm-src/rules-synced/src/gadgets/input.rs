use spring_addons::rules::Gadget;

use crate::input;
use crate::state::Global;

#[derive(Default)]
pub struct InputGadget;

impl InputGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for InputGadget {
    fn name(&self) -> &'static str {
        "Input"
    }

    fn handle_lua_msg(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        input::handle_message(ctx.global(), data);
    }
}
