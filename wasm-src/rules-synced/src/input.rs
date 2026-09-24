//! Player commands arriving from the UI over `RecvLuaMsg`.

use partsneeded_shared::{BodyPart, Command};
use spring::cmd;
use springcabal_control::protocol;

use crate::state::{FORM_COOLDOWN, Global};
use crate::triggers::nearby;

/// How close the player has to stand to pull a lever (ui_wisp_control.lua).
const LEVER_REACH: f32 = 100.0;

pub fn handle_message(global: &Global, data: &[u8]) {
    let Ok(message) = core::str::from_utf8(data) else {
        return;
    };
    if let Some(direction) = protocol::decode_direction(message) {
        steer(global, direction);
    } else if let Some(command) = Command::decode(message) {
        handle_command(global, command);
    }
}

/// The held movement keys.
pub fn steer(global: &Global, direction: (f32, f32)) {
    if global.endgame.access().game_over {
        return;
    }
    global.wisp.access_mut().steering.set(direction);
}

pub fn handle_command(global: &Global, command: Command) {
    if global.endgame.access().game_over {
        return;
    }
    let frame = crate::state::frame();
    match command {
        Command::SwitchForm => {
            let mut wisp = global.wisp.access_mut();
            // unit_wisp_controller.lua: without eyes a switch is refused and
            // doesn't start the cooldown either.
            if frame - wisp.form_frame <= FORM_COOLDOWN
                || !wisp.collected_parts.contains(BodyPart::Eyes)
            {
                return;
            }
            // The form-change sound is unsynced-only; the UI module plays it.
            wisp.form_frame = frame;
            wisp.spirit = !wisp.spirit;
            wisp.pieces_dirty = true;
        }
        Command::PullLever(id) => pull_lever(global, id),
    }
}

fn pull_lever(global: &Global, id: i32) {
    // The UI performs these checks too, but Lua messages are untrusted:
    // retain the original interaction contract in authoritative rules code.
    let (has_arms, spirit, wisp_id) = {
        let wisp = global.wisp.access();
        (
            wisp.collected_parts.contains(BodyPart::Arms),
            wisp.spirit,
            wisp.id,
        )
    };
    let is_lever = crate::defs::unit_def(id) == crate::defs::defs().lever;
    if spirit || !has_arms || !is_lever || !nearby(wisp_id, id, LEVER_REACH) {
        return;
    }
    let enabled = {
        let mut world = global.world.access_mut();
        let was_active = world.active_triggers.contains(&id);
        crate::triggers::set_trigger(&mut world, id, !was_active);
        !was_active
    };
    // The world borrow has ended: the order can raise callins synchronously.
    crate::log::check(
        "lever ONOFF order",
        id,
        spring::give_order_to_unit(id, cmd::ONOFF, &[f32::from(u8::from(enabled))], 0, 0),
    );
}
