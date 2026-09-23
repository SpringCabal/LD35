//! Player commands arriving from the UI over `RecvLuaMsg`.

use partsneeded_shared::BodyPart;

use crate::endgame;
use crate::state::{FORM_COOLDOWN, Global, param};
use crate::triggers::nearby;
use spring::cmd;

pub fn handle_message(global: &Global, data: &[u8]) {
    let Ok(message) = core::str::from_utf8(data) else {
        return;
    };
    {
        let mut fields = message.split('|');
        let command = fields.next();
        if global.endgame.access().game_over
            && matches!(command, Some("movement" | "switch_form" | "pull_lever"))
        {
            return;
        }
        let frame = crate::state::frame();
        match command {
            Some("movement") => {
                let x = fields
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0.0);
                let z = fields
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0.0);
                let mut wisp = global.wisp.access_mut();
                wisp.movement = (x != 0.0 || z != 0.0).then_some((x, z));
                wisp.movement_frame = frame;
                if wisp.movement.is_some() {
                    wisp.stop_issued = false;
                }
            }
            Some("switch_form") if frame - global.wisp.access().form_frame > FORM_COOLDOWN => {
                // The form-change sound is unsynced-only; the UI module plays it.
                let spirit = {
                    let mut wisp = global.wisp.access_mut();
                    wisp.form_frame = frame;
                    // Leaving spirit form needs eyes; otherwise the switch
                    // always returns to it.
                    wisp.spirit = !(wisp.spirit && wisp.collected_parts.contains(BodyPart::Eyes));
                    wisp.pieces_dirty = true;
                    wisp.spirit
                };
                param("spiritMode", if spirit { 1.0 } else { 0.0 });
            }
            Some("pull_lever") => {
                if let Some(id) = fields.next().and_then(|value| value.parse::<i32>().ok()) {
                    // The UI performs these checks too, but Lua messages are
                    // untrusted: retain the original interaction contract in
                    // authoritative rules code.
                    let (has_arms, spirit, wisp_id) = {
                        let wisp = global.wisp.access();
                        (
                            wisp.collected_parts.contains(BodyPart::Arms),
                            wisp.spirit,
                            wisp.id,
                        )
                    };
                    let is_lever = crate::defs::unit_def(id) == crate::defs::defs().lever;
                    if spirit || !has_arms || !is_lever || !nearby(wisp_id, id, 100.0) {
                        return;
                    }
                    let enabled = {
                        let mut world = global.world.access_mut();
                        if let Some(index) = world
                            .active_triggers
                            .iter()
                            .position(|active| *active == id)
                        {
                            world.active_triggers.remove(index);
                        } else {
                            world.active_triggers.push(id);
                        }
                        i32::from(world.active_triggers.contains(&id)) as f32
                    };
                    crate::log::check(
                        "lever ONOFF order",
                        id,
                        spring::give_order_to_unit(id, cmd::ONOFF, &[enabled], 0, 0),
                    );
                }
            }
            Some("npc_wisp") => {
                let save = matches!(fields.next(), Some("save"));
                if let Some(id) = fields.next().and_then(|value| value.parse::<i32>().ok()) {
                    endgame::mark_wisp_dying(global, id, save);
                }
            }
            _ => {}
        }
    }
}
