//! Drives the headless integration scenario through the real message path.

use partsneeded_shared::{BodyPart, Command};
use spring_addons::rules::Gadget;

use crate::defs::{defs, unit_def};
use crate::input::handle_command;
use crate::integration::{self, LAST_ROOM_PLATE, TEST_LEVER, TEST_PLATE};
use crate::state::Global;
use crate::triggers::{LEVER_GATE_COMBINATION, level_id};

const MOVE_RIGHT: Command = Command::Move { x: 1, z: 0 };
const STAND_STILL: Command = Command::Move { x: 0, z: 0 };

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
        match frame {
            20 => handle_command(global, MOVE_RIGHT),
            // Release the key again before the frame-30 check.
            26 => handle_command(global, STAND_STILL),
            // Keep the player moving right from here on, long enough for the
            // engine's turn rate to settle before the physical-form screenshot
            // at frame 86; game over stops it.
            73 => handle_command(global, MOVE_RIGHT),
            // Out of reach, and still a spirit: the rules must refuse it.
            25 => {
                if let Some(lever) = level_unit(global, TEST_LEVER) {
                    pull_lever(global, lever);
                }
            }
            35 => move_part_to_wisp(global, Some(BodyPart::Eyes)),
            61 => handle_command(global, Command::SwitchForm),
            70 => move_part_to_wisp(global, Some(BodyPart::Arms)),
            // Pull the same lever twice: the second pull has to toggle it off.
            72 | 78 => bring_and_pull_lever(global, TEST_LEVER),
            // Set the first door's lever combination.
            79 => {
                for (key, _) in LEVER_GATE_COMBINATION.iter().filter(|(_, on)| *on) {
                    bring_and_pull_lever(global, *key);
                }
            }
            80 => move_part_to_wisp(global, Some(BodyPart::Legs)),
            82 => {
                if let Some(plate) = level_unit(global, TEST_PLATE) {
                    move_wisp_next_to(global, plate, 45.0);
                }
            }
            86 | 88 | 90 => move_part_to_wisp(global, None),
            // Test the one-way last-room plate after the normal pickup and
            // game-over checks have completed, so this scenario cannot alter
            // the pickup sequence being tested above.
            110 => {
                if let Some(plate) = level_unit(global, LAST_ROOM_PLATE) {
                    move_wisp_next_to(global, plate, 0.0);
                }
            }
            116 => {
                let wisp = global.wisp.access().id;
                move_wisp_next_to(global, wisp, 180.0);
            }
            _ => {}
        }
    }
}

fn level_unit(global: &Global, key: i32) -> Option<i32> {
    level_id(&global.world.access(), key)
}

fn wisp_position(global: &Global) -> Option<spring::Float3> {
    spring::get_unit_position(global.wisp.access().id, false, false).ok()
}

fn pull_lever(global: &Global, lever: i32) {
    handle_command(global, Command::PullLever(lever));
}

fn bring_and_pull_lever(global: &Global, key: i32) {
    if let (Some(lever), Some(position)) = (level_unit(global, key), wisp_position(global)) {
        let _ = spring::set_unit_position(lever, position);
        pull_lever(global, lever);
    }
}

/// Place a body-part pickup on the player (`None`: the first one left).
fn move_part_to_wisp(global: &Global, part: Option<BodyPart>) {
    let pickup = {
        let wisp = global.wisp.access();
        match part {
            Some(part) => {
                let def = defs().part_def(part);
                wisp.body_parts
                    .iter()
                    .copied()
                    .find(|id| unit_def(*id) == def)
            }
            None => wisp.body_parts.first().copied(),
        }
    };
    if let (Some(pickup), Some(position)) = (pickup, wisp_position(global)) {
        let _ = spring::set_unit_position(pickup, position);
    }
}

/// Place the player `dx` elmos east of `unit`.
fn move_wisp_next_to(global: &Global, unit: i32, dx: f32) {
    let wisp = global.wisp.access().id;
    if let Ok(position) = spring::get_unit_position(unit, false, false) {
        let _ = spring::set_unit_position(
            wisp,
            spring::Float3::new(position.x + dx, position.y, position.z),
        );
    }
}
