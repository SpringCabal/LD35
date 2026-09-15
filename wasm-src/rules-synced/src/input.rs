//! Player commands arriving from the UI over `RecvLuaMsg`.

use crate::endgame;
use crate::state::{FORM_COOLDOWN, Global, param};
use crate::triggers::CMD_ONOFF;
use crate::triggers::nearby;

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
        let frame = global.clock.access().frame;
        match command {
            #[cfg(feature = "unit-script-probe")]
            Some("probe_begin") => {
                global.unit_script_probe.access_mut().active = true;
            }
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
                let wisp_id = {
                    let mut wisp = global.wisp.access_mut();
                    wisp.form_frame = frame;
                    // Leaving spirit form needs eyes; otherwise the switch
                    // always returns to it.
                    wisp.spirit = !(wisp.spirit && wisp.has_eyes);
                    wisp.pieces_dirty = true;
                    param("spiritMode", if wisp.spirit { 1.0 } else { 0.0 });
                    wisp.id
                };
                // unit_wisp_controller.lua plays this when the form actually
                // changes.  Playing it here does nothing -- sound is
                // unsynced-only -- so the UI module does it off the spirit flip.
                let _ = wisp_id;
            }
            Some("pull_lever") => {
                if let Some(id) = fields.next().and_then(|value| value.parse::<i32>().ok()) {
                    // The UI performs these checks too, but Lua messages are
                    // untrusted: retain the original interaction contract in
                    // authoritative rules code.
                    let (has_arms, spirit, wisp_id) = {
                        let wisp = global.wisp.access();
                        (
                            wisp.collected_parts.iter().any(|part| part == "arms"),
                            wisp.spirit,
                            wisp.id,
                        )
                    };
                    let is_lever =
                        spring::get_unit_def_id_by_name("lever")
                            .ok()
                            .is_some_and(|lever| {
                                spring::get_unit_def_id(id)
                                    .ok()
                                    .is_some_and(|def| def.0 == lever)
                            });
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
                    let _ = spring::give_order_to_unit(id, CMD_ONOFF, &[enabled], 0, 0);
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
