use alloc::format;
use alloc::vec::Vec;

use springcabal::check::Suite;

use crate::cages::OPEN_FRAMES;
use crate::gadgets::triggers::behavior::{is_trigger_active, level_id, nearby};
use crate::state::Global;

pub enum Step {
    CollectParts,
    SwitchForm,
    PullLever(i32),
    StandOn(i32),
}

struct Room {
    name: &'static str,
    lever: i32,
    plate: i32,
    cages: [(i32, bool); 4],
}

const ROOMS: [Room; 2] = [
    Room {
        name: "first room",
        lever: 27806,
        plate: 26492,
        cages: [(23261, false), (27501, false), (5316, false), (15054, true)],
    },
    Room {
        name: "right-hand room",
        lever: 18496,
        plate: 12688,
        cages: [
            (1811, false),
            (16824, false),
            (24489, false),
            (28033, false),
        ],
    },
];

const COLLECT_PARTS: i32 = 13;
const SWITCH_FORM: i32 = 15;
const FIRST_PASS_AFTER: i32 = 25;
const ROOM_GAP: i32 = 50;
const UPDATE_RATE: i32 = 5;
const PRISONER_RADIUS: f32 = 50.0;

const CHECKS: u32 = 5;

pub struct SamePassTracker {
    suite: Suite,
    first_pass: Option<i32>,
    prisoners: Vec<(i32, Option<i32>)>,
}

impl Default for SamePassTracker {
    fn default() -> Self {
        Self {
            suite: Suite::new(
                "[PartsNeeded-Rust-Test]",
                "PARTS NEEDED SAME-PASS INTEGRATION TEST",
                CHECKS,
            ),
            first_pass: None,
            prisoners: Vec::new(),
        }
    }
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut SamePassTracker,
    global: &Global,
    frame: i32,
) -> Option<Step> {
    if t.suite.is_finished() {
        return None;
    }
    let spawned = global.world.access().spawn_frame;
    let first_pass = *t.first_pass.get_or_insert_with(|| {
        let earliest = spawned + FIRST_PASS_AFTER;
        earliest + (UPDATE_RATE - earliest % UPDATE_RATE) % UPDATE_RATE
    });
    let after_spawn = frame - spawned;
    if after_spawn == COLLECT_PARTS {
        return Some(Step::CollectParts);
    }
    if after_spawn == SWITCH_FORM {
        return Some(Step::SwitchForm);
    }
    for (index, room) in ROOMS.iter().enumerate() {
        let pass = first_pass + index as i32 * ROOM_GAP;
        if frame == pass - UPDATE_RATE + 1 {
            return Some(Step::PullLever(room.lever));
        }
        if frame == pass - 1 {
            before_pass(spring, t, global, room);
            return Some(Step::StandOn(room.plate));
        }
        if frame == pass + 1 {
            after_pass(spring, t, global, room);
        }
        if index == 0 && frame == pass + OPEN_FRAMES + 1 {
            bars_open(spring, t, global, room);
        }
    }
    if frame == first_pass + (ROOMS.len() as i32 - 1) * ROOM_GAP + 1 {
        t.suite.finish(spring);
    }
    None
}

fn before_pass(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut SamePassTracker,
    global: &Global,
    room: &Room,
) {
    let wisp = global.wisp.access();
    let world = global.world.access();
    let key_active = |key| level_id(&world, key).is_some_and(|id| is_trigger_active(&world, id));
    let key_open =
        |key| level_id(&world, key).is_some_and(|id| world.active_triggerables.contains(&id));
    for (cage, _) in room.cages {
        let prisoner = level_id(&world, cage).and_then(|cage| {
            world
                .npc_wisps
                .iter()
                .copied()
                .find(|wisp| nearby(spring, cage, *wisp, PRISONER_RADIUS))
        });
        t.prisoners.push((cage, prisoner));
    }
    let embodied = !wisp.spirit
        && [
            partsneeded_shared::BodyPart::Arms,
            partsneeded_shared::BodyPart::Legs,
        ]
        .into_iter()
        .all(|part| wisp.collected_parts.contains(part));
    let lever = key_active(room.lever);
    let plate = key_active(room.plate);
    let open = room
        .cages
        .iter()
        .filter(|(cage, _)| key_open(*cage))
        .count();
    let caged = room
        .cages
        .iter()
        .filter(|(cage, _)| prisoner_of(t, *cage).is_some())
        .count();
    t.suite.check(spring,
        embodied && lever && !plate && open == 0 && caged == room.cages.len(),
        format!(
            "{}: before the pass the lever is on, the kill plate is up and all four cages are closed, each with its prisoner (body form with arms and legs: {embodied}, lever {lever}, plate {plate}, {open} open, {caged} caged)",
            room.name
        ),
    );
}

fn after_pass(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut SamePassTracker,
    global: &Global,
    room: &Room,
) {
    let world = global.world.access();
    let endgame = global.endgame.access();
    let plate = level_id(&world, room.plate).is_some_and(|id| is_trigger_active(&world, id));
    let open = room
        .cages
        .iter()
        .filter(|(cage, _)| {
            level_id(&world, *cage).is_some_and(|id| world.active_triggerables.contains(&id))
        })
        .count();
    let fates: Vec<(i32, Option<bool>)> = room
        .cages
        .iter()
        .map(|(cage, _)| (*cage, fate(t, &endgame.dying_wisps, *cage)))
        .collect();
    let as_expected = room
        .cages
        .iter()
        .zip(&fates)
        .all(|((_, saved), (_, fate))| *fate == if *saved { None } else { Some(false) });
    t.suite.check(spring,
        plate && open == room.cages.len() && as_expected,
        format!(
            "{}: stepping on the kill plate opens all four cages in the pass that also sees the lever, and the link listed first decides each prisoner's fate (plate {plate}, {open} open, killed at once: {:?})",
            room.name,
            fates
                .iter()
                .map(|(cage, fate)| (*cage, *fate == Some(false)))
                .collect::<Vec<_>>()
        ),
    );
}

fn bars_open(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut SamePassTracker,
    global: &Global,
    room: &Room,
) {
    let endgame = global.endgame.access();
    let fates: Vec<(i32, Option<bool>)> = room
        .cages
        .iter()
        .map(|(cage, _)| (*cage, fate(t, &endgame.dying_wisps, *cage)))
        .collect();
    let as_expected = room
        .cages
        .iter()
        .zip(&fates)
        .all(|((_, saved), (_, fate))| *fate == Some(*saved));
    t.suite.check(spring,
        as_expected,
        format!(
            "{}: the lever's link, listed before the kill plate's, frees cage 15054's prisoner once its bars are open (saved: {:?})",
            room.name, fates
        ),
    );
}

fn prisoner_of(t: &SamePassTracker, cage: i32) -> Option<i32> {
    t.prisoners
        .iter()
        .find(|(key, _)| *key == cage)
        .and_then(|(_, prisoner)| *prisoner)
}

fn fate(t: &SamePassTracker, dying: &[(i32, bool, i32)], cage: i32) -> Option<bool> {
    let prisoner = prisoner_of(t, cage)?;
    dying
        .iter()
        .find(|(id, _, _)| *id == prisoner)
        .map(|(_, saved, _)| *saved)
}
