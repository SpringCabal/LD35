//! The restart from the end screen, after the ending scenario: the UI's
//! headless-integration build asks for it (`ui/src/widgets/dbg_restart.rs`),
//! and the level must come back as the first round had it, the same number
//! of frames after its spawn.

use alloc::format;
use alloc::vec::Vec;

use partsneeded_shared::BodyPart;
use springcabal::check::Suite;

use crate::defs::{defs, unit_def};
use crate::state::Global;
use crate::triggers::{GATE_AREAS, GATE_CLOSED_HEIGHT};

/// When a round is compared, frames after its level spawned: the main
/// scenario's first check.
const AFTER_SPAWN: i32 = 12;

/// The first gate, opened by its levers, and the last one, opened by its
/// plate, in the first round.
const OPENED_GATES: [i32; 2] = [20, 23];

/// What the level looks like `AFTER_SPAWN` frames after it spawned.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Census {
    units: usize,
    /// Units per def the level spawns: player, NPC wisps, gates, levers,
    /// plates, cages, lights and the six pickups.
    per_def: Vec<usize>,
}

fn census() -> Census {
    let units: Vec<i32> = spring::get_all_units().unwrap_or_default();
    let defs = defs();
    let tracked = [
        defs.wisp,
        defs.npcwisp,
        defs.gate,
        defs.lever,
        defs.plate,
        defs.cage,
        defs.light,
    ]
    .into_iter()
    .chain(BodyPart::ALL.map(|part| defs.part_def(part)));
    Census {
        units: units.len(),
        per_def: tracked
            .map(|def| units.iter().filter(|id| unit_def(**id) == def).count())
            .collect(),
    }
}

pub struct RestartTracker {
    suite: Suite,
    first_round: Option<Census>,
}

impl Default for RestartTracker {
    fn default() -> Self {
        Self {
            suite: Suite::new(
                "[PartsNeeded-Rust-Test]",
                "PARTS NEEDED RESTART INTEGRATION TEST",
            )
            .require_at_least(4),
            first_round: None,
        }
    }
}

pub fn update(t: &mut RestartTracker, global: &Global, frame: i32) {
    if t.suite.is_finished() {
        return;
    }
    let spawned = global.world.access().spawn_frame;
    if frame != spawned + AFTER_SPAWN {
        return;
    }
    match t.first_round.clone() {
        None => t.first_round = Some(census()),
        Some(first) => check_restarted(t, global, &first),
    }
}

fn check_restarted(t: &mut RestartTracker, global: &Global, first: &Census) {
    let now = census();
    t.suite.check(
        now == *first,
        format!(
            "the restart builds the level again as the first round had it ({} units, per def {:?}; first round {} units, {:?})",
            now.units, now.per_def, first.units, first.per_def
        ),
    );
    let (spirit, parts, game_over, triggered, npc_wisps, level_units) = {
        let wisp = global.wisp.access();
        let world = global.world.access();
        let endgame = global.endgame.access();
        (
            wisp.spirit,
            wisp.collected_parts,
            endgame.game_over || endgame.ending.is_some(),
            world.active_triggers.len() + world.active_triggerables.len(),
            world.npc_wisps.len(),
            world.level_ids.len(),
        )
    };
    t.suite.check(
        spirit && parts.is_empty() && !game_over && triggered == 0,
        format!(
            "the restarted round starts over: a spirit with no parts, the game running, nothing triggered (spirit {spirit}, parts {parts:?}, over {game_over}, {triggered} triggered)"
        ),
    );
    t.suite.check(
        npc_wisps == 10 && level_units == 61,
        format!(
            "the restarted round tracks the new level: 61 level units and 10 caged NPC wisps (got {level_units} and {npc_wisps})"
        ),
    );
    let heights: Vec<f32> = GATE_AREAS
        .iter()
        .filter(|(key, ..)| OPENED_GATES.contains(key))
        .map(|(_, x1, z1, x2, z2)| {
            spring::get_ground_height((x1 + x2) / 2.0, (z1 + z2) / 2.0).unwrap_or(0.0)
        })
        .collect();
    t.suite.check(
        heights.len() == OPENED_GATES.len()
            && heights
                .iter()
                .all(|height| (height - GATE_CLOSED_HEIGHT).abs() < 1.0),
        format!(
            "the restart closes the gates the first round opened: the doorway terrain is back at {GATE_CLOSED_HEIGHT} (got {heights:?})"
        ),
    );
    t.suite.finish();
}
