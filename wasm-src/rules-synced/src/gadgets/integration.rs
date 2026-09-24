//! Drives the headless integration scenario through the real message path.

use core::cell::{Cell, RefCell};

use partsneeded_shared::{BodyPart, Command};
use spring_addons::rules::Gadget;

use crate::defs::{defs, unit_def};
use crate::input::{handle_message, steer};
use crate::integration::scenario::{KILL_PLATE, LAST_ROOM_PLATE, TEST_LEVER, TEST_PLATE};
use crate::integration::{self, Tracker, ending};
use crate::state::Global;
use crate::triggers::{LEVER_GATE_COMBINATION, level_id};

const MOVE_RIGHT: (f32, f32) = (1.0, 0.0);
const STAND_STILL: (f32, f32) = (0.0, 0.0);

#[derive(Default)]
pub struct IntegrationGadget {
    tracker: RefCell<Tracker>,
    ending: RefCell<ending::EndingTracker>,
    /// Whether the UI module's hello arrived over RecvLuaMsg.
    ui_hello: Cell<bool>,
    /// Where the player stood before the kill-plate step, restored for the
    /// ending scenario.
    before_kill_plate: Cell<Option<spring::Float3>>,
}

impl IntegrationGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn handle_lua_msg(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if data == Command::Hello.encode().as_bytes() {
            self.ui_hello.set(true);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        {
            let mut tracker = self.tracker.borrow_mut();
            tracker.ui_hello = self.ui_hello.get();
            integration::update(&mut tracker, global, frame);
        }
        ending::update(&mut self.ending.borrow_mut(), global, frame);
        match frame {
            20 => steer(global, MOVE_RIGHT),
            // Release the key again before the frame-30 check.
            26 => steer(global, STAND_STILL),
            // Keep the player moving right from here on, long enough for the
            // engine's turn rate to settle before the physical-form screenshot
            // at frame 86; game over stops it.
            73 => steer(global, MOVE_RIGHT),
            // Out of reach, and still a spirit: the rules must refuse it.
            25 => {
                if let Some(lever) = level_unit(global, TEST_LEVER) {
                    pull_lever(global, lever);
                }
            }
            // Past the cooldown from frame 0, still without eyes: refused, and
            // it must not start the cooldown that would refuse the next one.
            62 => send(global, Command::SwitchForm),
            63 => move_part_to_wisp(global, Some(BodyPart::Eyes)),
            66 => send(global, Command::SwitchForm),
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
            // After game over, and past the cooldown from frame 66: refused.
            130 => send(global, Command::SwitchForm),
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
            146 => {
                self.before_kill_plate.set(wisp_position(global));
                if let Some(plate) = level_unit(global, KILL_PLATE) {
                    move_wisp_next_to(global, plate, 0.0);
                }
            }
            160 => {
                if let Some(position) = self.before_kill_plate.get() {
                    let _ = spring::set_unit_position(global.wisp.access().id, position);
                }
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

/// Deliver a command the way RecvLuaMsg does.
fn send(global: &Global, command: Command) {
    handle_message(global, command.encode().as_bytes());
}

fn pull_lever(global: &Global, lever: i32) {
    send(global, Command::PullLever(lever));
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
