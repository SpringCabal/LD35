use alloc::rc::Rc;
use alloc::string::String;
use core::cell::{Cell, RefCell};

use partsneeded_shared::{BodyPart, Command};

use spring_addons::rules::Gadget;

use crate::defs::{defs, unit_def};
use crate::gadgets::triggers::behavior::level_id;
use crate::integration::same_pass::{self, Step};
use crate::integration::scenario::{
    KILL_PLATE, LAST_ROOM_PLATE, LEVER_GATE_PULLS, TEST_LEVER, TEST_PLATE,
};
use crate::integration::{self, Tracker, ending, restart};
use crate::state::Global;
use springcabal_control::protocol;

const MOVE_RIGHT: (i32, i32) = (1, 0);
const STAND_STILL: (i32, i32) = (0, 0);

#[derive(Default)]
pub struct IntegrationGadget {
    tracker: RefCell<Tracker>,
    ending: RefCell<ending::EndingTracker>,
    restart: RefCell<restart::RestartTracker>,
    same_pass: RefCell<same_pass::SamePassTracker>,
    sent: Rc<Cell<u32>>,
    received: Cell<u32>,
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
        let spring = _ctx.spring();
        if data == Command::Hello.encode().as_bytes() {
            let _ = spring.messages().echo(
                "[PartsNeeded-Rust-Test] PARTS NEEDED UI HANDSHAKE PASSED",
                "",
            );
        } else if core::str::from_utf8(data).is_ok_and(|message| {
            Command::decode(message).is_some() || protocol::decode_direction(message).is_some()
        }) {
            self.received.set(self.received.get() + 1);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        {
            let mut tracker = self.tracker.borrow_mut();
            tracker.messages = (self.sent.get(), self.received.get());
            integration::update(spring, &mut tracker, global, frame);
        }
        ending::update(spring, &mut self.ending.borrow_mut(), global, frame);
        restart::update(spring, &mut self.restart.borrow_mut(), global, frame);
        if self.restart.borrow().is_finished() {
            let step = same_pass::update(spring, &mut self.same_pass.borrow_mut(), global, frame);
            if let Some(step) = step {
                self.same_pass_step(spring, global, step);
            }
        }
        match frame {
            20 => self.steer(MOVE_RIGHT),
            26 => self.steer(STAND_STILL),
            73 => self.steer(MOVE_RIGHT),
            91 => self.steer(STAND_STILL),
            25 => {
                if let Some(lever) = level_unit(global, TEST_LEVER) {
                    self.send(Command::PullLever(lever));
                }
            }
            62 => self.send(Command::SwitchForm),
            63 => move_part_to_wisp(spring, global, Some(BodyPart::Eyes)),
            66 => self.send(Command::SwitchForm),
            70 => move_part_to_wisp(spring, global, Some(BodyPart::Arms)),
            72 | 78 => self.bring_and_pull_lever(spring, global, TEST_LEVER),
            79 => {
                for key in LEVER_GATE_PULLS {
                    self.bring_and_pull_lever(spring, global, key);
                }
            }
            80 => move_part_to_wisp(spring, global, Some(BodyPart::Legs)),
            82 => {
                if let Some(plate) = level_unit(global, TEST_PLATE) {
                    move_wisp_next_to(spring, global, plate, 45.0);
                }
            }
            124 | 126 | 138 => move_part_to_wisp(spring, global, None),
            146 => {
                if let Some(plate) = level_unit(global, LAST_ROOM_PLATE) {
                    move_wisp_next_to(spring, global, plate, 0.0);
                }
            }
            152 => {
                let wisp = global.wisp.access().id;
                move_wisp_next_to(spring, global, wisp, 180.0);
            }
            160 => self.send(Command::SwitchForm),
            166 => {
                self.before_kill_plate.set(wisp_position(spring, global));
                if let Some(plate) = level_unit(global, KILL_PLATE) {
                    move_wisp_next_to(spring, global, plate, 0.0);
                }
            }
            180 => {
                if let Some(position) = self.before_kill_plate.get() {
                    let _ = crate::with_addon_context(|ctx| {
                        ctx.commands()
                            .set_unit_position(global.wisp.access().id, position)
                    });
                }
            }
            _ => {}
        }
    }
}

fn level_unit(global: &Global, key: i32) -> Option<i32> {
    level_id(&global.world.access(), key)
}

fn wisp_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
) -> Option<spring::Float3> {
    spring
        .units_info()
        .get_unit_position(global.wisp.access().id, Default::default())
        .ok()
}

impl IntegrationGadget {
    fn send_message(&self, message: &str) {
        let message = String::from(message);
        let sent = Rc::clone(&self.sent);
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .queue(move |spring| springcabal::message::send_to_rules(spring, &message))
        })
        .then(move |_spring, delivered| {
            if delivered {
                sent.set(sent.get() + 1);
            }
        });
    }

    fn send(&self, command: Command) {
        self.send_message(&command.encode());
    }

    fn steer(&self, direction: (i32, i32)) {
        self.send_message(&protocol::encode_direction(direction));
    }

    fn same_pass_step(
        &self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        step: Step,
    ) {
        match step {
            Step::CollectParts => {
                for part in [BodyPart::Eyes, BodyPart::Arms, BodyPart::Legs] {
                    move_part_to_wisp(spring, global, Some(part));
                }
            }
            Step::SwitchForm => self.send(Command::SwitchForm),
            Step::PullLever(key) => self.bring_and_pull_lever(spring, global, key),
            Step::StandOn(key) => {
                if let Some(plate) = level_unit(global, key) {
                    move_wisp_next_to(spring, global, plate, 0.0);
                }
            }
        }
    }

    fn bring_and_pull_lever(
        &self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        key: i32,
    ) {
        if let (Some(lever), Some(position)) =
            (level_unit(global, key), wisp_position(spring, global))
        {
            let _ =
                crate::with_addon_context(|ctx| ctx.commands().set_unit_position(lever, position));
            self.send(Command::PullLever(lever));
        }
    }
}

fn move_part_to_wisp(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    part: Option<BodyPart>,
) {
    let pickup = {
        let wisp = global.wisp.access();
        match part {
            Some(part) => {
                let def = defs(spring).part_def(part);
                wisp.body_parts
                    .iter()
                    .copied()
                    .find(|id| unit_def(spring, *id) == def)
            }
            None => wisp.body_parts.first().copied(),
        }
    };
    if let (Some(pickup), Some(position)) = (pickup, wisp_position(spring, global)) {
        let _ = crate::with_addon_context(|ctx| ctx.commands().set_unit_position(pickup, position));
    }
}

fn move_wisp_next_to(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit: i32,
    dx: f32,
) {
    let wisp = global.wisp.access().id;
    if let Ok(position) = spring
        .units_info()
        .get_unit_position(unit, Default::default())
    {
        let _ = crate::with_addon_context(|ctx| {
            ctx.commands().set_unit_position(
                wisp,
                spring::Float3::new(position.x + dx, position.y, position.z),
            )
        });
    }
}
