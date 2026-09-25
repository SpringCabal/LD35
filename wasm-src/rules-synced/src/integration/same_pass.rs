//! A lever and a kill plate opening the same cage in one trigger pass, run in
//! the restarted round once the restart suite is done, where every cage is
//! closed again.
//!
//! cage.lua reads `do_kill` when the cage activates, so the first link that
//! opens it decides its prisoner's fate. The port walks the links in
//! game_custom_spawn.lua's table order: in the first room the levers are
//! listed before the kill plate, in the right-hand room the kill plates
//! before the lever. Each room is played once: the lever is pulled between
//! two passes, and the player steps on the plate on the frame before the
//! next one.

use alloc::format;
use alloc::vec::Vec;

use springcabal::check::Suite;

use crate::cages::OPEN_FRAMES;
use crate::state::Global;
use crate::triggers::{is_trigger_active, level_id, nearby};

/// What the scenario asks the integration gadget to do on a frame.
pub enum Step {
    /// Put the eyes, arms and legs pickups on the player.
    CollectParts,
    SwitchForm,
    /// Bring the lever with this level key to the player and pull it.
    PullLever(i32),
    /// Put the player on the plate with this level key.
    StandOn(i32),
}

/// A room where a lever and a kill plate both open cages. The keys and
/// fates are read from game_custom_spawn.lua, not from the rules' table, so
/// a mistake there can't make the check pass.
struct Room {
    name: &'static str,
    lever: i32,
    plate: i32,
    /// The plate's cages, each with whether its prisoner is saved.
    cages: [(i32, bool); 4],
}

const ROOMS: [Room; 2] = [
    // `[27806] = 15054` is listed before `[26492] = { -23261, -27501,
    // -5316, -15054 }`: the lever's link comes first.
    Room {
        name: "first room",
        lever: 27806,
        plate: 26492,
        cages: [(23261, false), (27501, false), (5316, false), (15054, true)],
    },
    // `[12688] = { -1811, -16824, -24489, -28033 }` is listed before
    // `[18496] = { 1811, 16824, 24489, 28033 }`: the kill plate's links come
    // first.
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

/// Frames after the restarted level spawned: the restart suite's checks are
/// made on 12.
const COLLECT_PARTS: i32 = 13;
const SWITCH_FORM: i32 = 15;
/// The first room's pass is the first trigger pass at least this late.
const FIRST_PASS_AFTER: i32 = 25;
/// Frames between the two rooms' passes: the first room's saved prisoner is
/// checked on the pass frame plus `OPEN_FRAMES + 1`.
const ROOM_GAP: i32 = 50;
/// unit_plate.lua's UPDATE_RATE, as `gadgets::triggers` uses it.
const UPDATE_RATE: i32 = 5;
/// cage.lua looks for its prisoner within this radius.
const PRISONER_RADIUS: f32 = 50.0;

/// Each room makes a precondition check and a same-pass check; the first
/// room also checks the saved prisoner once its bars are open.
const CHECKS: u32 = 5;

pub struct SamePassTracker {
    suite: Suite,
    /// The first room's pass frame, fixed once the restarted round is seen.
    first_pass: Option<i32>,
    /// Each cage's prisoner, found on the frame before its room's pass.
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

/// Run the scenario for `frame`, after the restart suite has finished.
/// Returns what the integration gadget has to do on this frame.
pub fn update(t: &mut SamePassTracker, global: &Global, frame: i32) -> Option<Step> {
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
        // Between two passes, so the lever is on before the plate's pass.
        if frame == pass - UPDATE_RATE + 1 {
            return Some(Step::PullLever(room.lever));
        }
        if frame == pass - 1 {
            before_pass(t, global, room);
            return Some(Step::StandOn(room.plate));
        }
        if frame == pass + 1 {
            after_pass(t, global, room);
        }
        if index == 0 && frame == pass + OPEN_FRAMES + 1 {
            bars_open(t, global, room);
        }
    }
    if frame == first_pass + (ROOMS.len() as i32 - 1) * ROOM_GAP + 1 {
        t.suite.finish();
    }
    None
}

fn before_pass(t: &mut SamePassTracker, global: &Global, room: &Room) {
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
                .find(|wisp| nearby(cage, *wisp, PRISONER_RADIUS))
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
    t.suite.check(
        embodied && lever && !plate && open == 0 && caged == room.cages.len(),
        format!(
            "{}: before the pass the lever is on, the kill plate is up and all four cages are closed, each with its prisoner (body form with arms and legs: {embodied}, lever {lever}, plate {plate}, {open} open, {caged} caged)",
            room.name
        ),
    );
}

fn after_pass(t: &mut SamePassTracker, global: &Global, room: &Room) {
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
    // A killing cage resolves at once; a saving one only once its bars are
    // open, 40 frames later.
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
    t.suite.check(
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

fn bars_open(t: &mut SamePassTracker, global: &Global, room: &Room) {
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
    t.suite.check(
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

/// Whether the cage's prisoner is being saved (`Some(true)`), killed
/// (`Some(false)`), or neither yet.
fn fate(t: &SamePassTracker, dying: &[(i32, bool, i32)], cage: i32) -> Option<bool> {
    let prisoner = prisoner_of(t, cage)?;
    dying
        .iter()
        .find(|(id, _, _)| *id == prisoner)
        .map(|(_, saved, _)| *saved)
}
