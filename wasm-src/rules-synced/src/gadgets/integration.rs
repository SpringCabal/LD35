//! Drives the headless integration scenario through the real message path.

use core::cell::{Cell, RefCell};

use partsneeded_shared::{BodyPart, Command};
use spring_addons::rules::Gadget;

use crate::defs::{defs, unit_def};
use crate::integration::scenario::{
    KILL_PLATE, LAST_ROOM_PLATE, LEVER_GATE_PULLS, TEST_LEVER, TEST_PLATE,
};
use crate::integration::{self, Tracker, ending, restart};
use crate::state::Global;
use crate::triggers::level_id;
use springcabal_control::protocol;

const MOVE_RIGHT: (i32, i32) = (1, 0);
const STAND_STILL: (i32, i32) = (0, 0);

#[derive(Default)]
pub struct IntegrationGadget {
    tracker: RefCell<Tracker>,
    ending: RefCell<ending::EndingTracker>,
    restart: RefCell<restart::RestartTracker>,
    /// Whether the UI module's hello arrived over RecvLuaMsg.
    ui_hello: Cell<bool>,
    /// The scenario's own commands and movement messages: how many it sent,
    /// and how many came back over RecvLuaMsg.
    sent: Cell<u32>,
    received: Cell<u32>,
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
        } else if core::str::from_utf8(data).is_ok_and(|message| {
            Command::decode(message).is_some() || protocol::decode_direction(message).is_some()
        }) {
            self.received.set(self.received.get() + 1);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        {
            let mut tracker = self.tracker.borrow_mut();
            tracker.ui_hello = self.ui_hello.get();
            tracker.messages = (self.sent.get(), self.received.get());
            integration::update(&mut tracker, global, frame);
        }
        ending::update(&mut self.ending.borrow_mut(), global, frame);
        restart::update(&mut self.restart.borrow_mut(), global, frame);
        match frame {
            20 => self.steer(MOVE_RIGHT),
            // Release the key again before the frame-30 check.
            26 => self.steer(STAND_STILL),
            // Walk right, from the start room and on the plate it is moved
            // to at 82, then stand still on the plate for the visual suite's
            // settled shots 3 and 4 (ui/src/widgets/dbg_e2e.rs).
            73 => self.steer(MOVE_RIGHT),
            91 => self.steer(STAND_STILL),
            // Out of reach, and still a spirit: the rules must refuse it.
            25 => {
                if let Some(lever) = level_unit(global, TEST_LEVER) {
                    self.send(Command::PullLever(lever));
                }
            }
            // Past the cooldown from frame 0, still without eyes: refused, and
            // it must not start the cooldown that would refuse the next one.
            62 => self.send(Command::SwitchForm),
            63 => move_part_to_wisp(global, Some(BodyPart::Eyes)),
            66 => self.send(Command::SwitchForm),
            70 => move_part_to_wisp(global, Some(BodyPart::Arms)),
            // Pull the same lever twice: the second pull has to toggle it off.
            72 | 78 => self.bring_and_pull_lever(global, TEST_LEVER),
            // Set the first door's lever combination.
            79 => {
                for key in LEVER_GATE_PULLS {
                    self.bring_and_pull_lever(global, key);
                }
            }
            80 => move_part_to_wisp(global, Some(BodyPart::Legs)),
            82 => {
                if let Some(plate) = level_unit(global, TEST_PLATE) {
                    move_wisp_next_to(global, plate, 45.0);
                }
            }
            // The last three parts: two after shot 3, one after shot 4.
            124 | 126 | 138 => move_part_to_wisp(global, None),
            // Test the one-way last-room plate after the normal pickup and
            // game-over checks have completed, so this scenario cannot alter
            // the pickup sequence being tested above.
            146 => {
                if let Some(plate) = level_unit(global, LAST_ROOM_PLATE) {
                    move_wisp_next_to(global, plate, 0.0);
                }
            }
            152 => {
                let wisp = global.wisp.access().id;
                move_wisp_next_to(global, wisp, 180.0);
            }
            // After game over, and past the cooldown from frame 66: refused.
            160 => self.send(Command::SwitchForm),
            166 => {
                self.before_kill_plate.set(wisp_position(global));
                if let Some(plate) = level_unit(global, KILL_PLATE) {
                    move_wisp_next_to(global, plate, 0.0);
                }
            }
            180 => {
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

impl IntegrationGadget {
    /// Send a message the way the UI does: over the network, back to the
    /// rules' RecvLuaMsg.
    fn send_message(&self, message: &str) {
        match spring::send_lua_rules_msg(message) {
            Ok(_) => self.sent.set(self.sent.get() + 1),
            Err(error) => {
                spring::log_warning!(crate::log::SECTION, "sending {message} failed: {error:?}")
            }
        }
    }

    fn send(&self, command: Command) {
        self.send_message(&command.encode());
    }

    /// The held movement keys, as the UI reports them.
    fn steer(&self, direction: (i32, i32)) {
        self.send_message(&protocol::encode_direction(direction));
    }

    fn bring_and_pull_lever(&self, global: &Global, key: i32) {
        if let (Some(lever), Some(position)) = (level_unit(global, key), wisp_position(global)) {
            let _ = spring::set_unit_position(lever, position);
            self.send(Command::PullLever(lever));
        }
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
