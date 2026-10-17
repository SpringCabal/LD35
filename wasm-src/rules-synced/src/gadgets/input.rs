use partsneeded_shared::{BodyPart, Command, LEVER_REACH};
use spring::cmd;
use springcabal_control::protocol;

use crate::gadgets::triggers::behavior::nearby;
use crate::state::{FORM_COOLDOWN, Global};

use spring_addons::rules::Gadget;

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
        let spring = ctx.spring();
        let global = ctx.global();
        let Ok(message) = core::str::from_utf8(data) else {
            return;
        };
        if let Some(direction) = protocol::decode_direction(message) {
            steer(global, direction);
        } else if let Some(command) = Command::decode(message) {
            handle_command(spring, global, command);
        }
    }
}

fn steer(global: &Global, direction: (f32, f32)) {
    if global.endgame.access().game_over {
        return;
    }
    global.wisp.access_mut().steering.set(direction);
}

fn handle_command(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    command: Command,
) {
    if global.endgame.access().game_over {
        return;
    }
    let frame = spring.game().current_frame().unwrap_or(0);
    match command {
        Command::SwitchForm => {
            let mut wisp = global.wisp.access_mut();
            if frame - wisp.form_frame <= FORM_COOLDOWN
                || !wisp.collected_parts.contains(BodyPart::Eyes)
            {
                return;
            }
            wisp.form_frame = frame;
            wisp.spirit = !wisp.spirit;
            wisp.pieces_dirty = true;
        }
        Command::PullLever(id) => pull_lever(spring, global, id),
        Command::Hello | Command::Restart => {}
    }
}

fn pull_lever(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, id: i32) {
    let (has_arms, spirit, wisp_id) = {
        let wisp = global.wisp.access();
        (
            wisp.collected_parts.contains(BodyPart::Arms),
            wisp.spirit,
            wisp.id,
        )
    };
    let is_lever = crate::defs::unit_def(spring, id) == crate::defs::defs(spring).lever;
    if spirit || !has_arms || !is_lever || !nearby(spring, wisp_id, id, LEVER_REACH) {
        return;
    }
    let enabled = {
        let mut world = global.world.access_mut();
        let enabled = !world.active_triggers.contains(&id);
        crate::gadgets::triggers::behavior::set_trigger(&mut world, id, enabled);
        enabled
    };
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .give_order_to_unit(id, cmd::ONOFF, &[f32::from(u8::from(enabled))], 0, 0)
    })
    .then(move |spring, result| {
        crate::log::check(spring, "lever ONOFF order", id, result);
    });
}
