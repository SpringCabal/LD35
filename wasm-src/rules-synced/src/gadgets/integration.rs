use spring_addons::rules::Gadget;

use crate::integration;
use crate::state::Global;

#[derive(Default)]
pub struct IntegrationGadget;

impl IntegrationGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        integration::update(global, frame);
        integration::update_ending_integration(global, frame);
        // Keep the player moving right long enough for the engine's turn rate
        // to settle before the physical-form screenshot at frame 86.
        if (73..=86).contains(&frame) {
            crate::input::handle_message(global, b"movement|1|0");
        }
        match frame {
            20 => crate::input::handle_message(global, b"movement|1|0"),
            25 => {
                let lever = crate::triggers::level_id(&global.world.access(), 14725);
                if let Some(lever) = lever {
                    let message = alloc::format!("pull_lever|{lever}");
                    crate::input::handle_message(global, message.as_bytes());
                }
            }
            35 => {
                let target = (|| {
                    let state = global.wisp.access();
                    let eyes = spring::get_unit_def_id_by_name("eyes").ok()?;
                    let body_part = state.body_parts.iter().copied().find(|id| {
                        spring::get_unit_def_id(*id)
                            .ok()
                            .is_some_and(|def| def.0 == eyes)
                    })?;
                    let position = spring::get_unit_position(state.id, false, false).ok()?;
                    Some((body_part, position))
                })();
                if let Some((body_part, position)) = target {
                    let _ = spring::set_unit_position(body_part, position);
                }
            }
            61 => crate::input::handle_message(global, b"switch_form"),
            70 => {
                let target = (|| {
                    let state = global.wisp.access();
                    let arms = spring::get_unit_def_id_by_name("arms").ok()?;
                    let body_part = state.body_parts.iter().copied().find(|id| {
                        spring::get_unit_def_id(*id)
                            .ok()
                            .is_some_and(|def| def.0 == arms)
                    })?;
                    let position = spring::get_unit_position(state.id, false, false).ok()?;
                    Some((body_part, position))
                })();
                if let Some((body_part, position)) = target {
                    let _ = spring::set_unit_position(body_part, position);
                }
            }
            72 => {
                let target = (|| {
                    Some((
                        crate::triggers::level_id(&global.world.access(), 14725)?,
                        spring::get_unit_position(global.wisp.access().id, false, false).ok()?,
                    ))
                })();
                if let Some((lever, position)) = target {
                    let _ = spring::set_unit_position(lever, position);
                    let message = alloc::format!("pull_lever|{lever}");
                    crate::input::handle_message(global, message.as_bytes());
                }
            }
            // Pull the same lever a second time: it has to toggle back off.
            78 => {
                let target = (|| {
                    Some((
                        crate::triggers::level_id(&global.world.access(), 14725)?,
                        spring::get_unit_position(global.wisp.access().id, false, false).ok()?,
                    ))
                })();
                if let Some((lever, position)) = target {
                    let _ = spring::set_unit_position(lever, position);
                    let message = alloc::format!("pull_lever|{lever}");
                    crate::input::handle_message(global, message.as_bytes());
                }
            }
            // The first door's lever combination.  game_custom_spawn.lua asks
            // for 29318, 14619 and 19115 ON and 23884 left OFF -- the floor
            // markers in that room spell out the same three circles and one
            // cross.
            79 => {
                for key in [29318, 14619, 19115] {
                    let target = (|| {
                        Some((
                            crate::triggers::level_id(&global.world.access(), key)?,
                            spring::get_unit_position(global.wisp.access().id, false, false)
                                .ok()?,
                        ))
                    })();
                    if let Some((lever, position)) = target {
                        let _ = spring::set_unit_position(lever, position);
                        let message = alloc::format!("pull_lever|{lever}");
                        crate::input::handle_message(global, message.as_bytes());
                    }
                }
            }
            80 => move_named_part_to_wisp(global, "legs"),
            82 => {
                let target = (|| {
                    Some((
                        global.wisp.access().id,
                        crate::triggers::level_id(&global.world.access(), 12701)?,
                    ))
                })();
                if let Some((wisp, plate)) = target
                    && let Ok(position) = spring::get_unit_position(plate, false, false)
                {
                    let _ = spring::set_unit_position(
                        wisp,
                        spring::Float3::new(position.x + 45.0, position.y, position.z),
                    );
                }
            }
            86 | 88 | 90 => {
                let target = (|| {
                    let state = global.wisp.access();
                    let body_part = state.body_parts.first().copied()?;
                    let position = spring::get_unit_position(state.id, false, false).ok()?;
                    Some((body_part, position))
                })();
                if let Some((body_part, position)) = target {
                    let _ = spring::set_unit_position(body_part, position);
                }
            }
            // Test the one-way last-room plate after the normal pickup and
            // game-over checks have completed, so this scenario cannot alter
            // the pickup sequence being tested above.
            110 => {
                let wisp = global.wisp.access().id;
                if let Some(plate) = crate::triggers::level_id(&global.world.access(), 11688)
                    && let Ok(position) = spring::get_unit_position(plate, false, false)
                {
                    let _ = spring::set_unit_position(wisp, position);
                }
            }
            116 => {
                let wisp = global.wisp.access().id;
                if let Ok(position) = spring::get_unit_position(wisp, false, false) {
                    let _ = spring::set_unit_position(
                        wisp,
                        spring::Float3::new(position.x + 180.0, position.y, position.z),
                    );
                }
            }
            _ => {}
        }
    }
}

fn move_named_part_to_wisp(global: &crate::state::Global, name: &str) {
    let target = (|| {
        let state = global.wisp.access();
        let def = spring::get_unit_def_id_by_name(name).ok()?;
        let body_part = state.body_parts.iter().copied().find(|id| {
            spring::get_unit_def_id(*id)
                .ok()
                .is_some_and(|candidate| candidate.0 == def)
        })?;
        let position = spring::get_unit_position(state.id, false, false).ok()?;
        Some((body_part, position))
    })();
    if let Some((body_part, position)) = target {
        let _ = spring::set_unit_position(body_part, position);
    }
}
