use alloc::format;
use alloc::vec::Vec;

use partsneeded_shared::BodyPart;
use springcabal::check::Suite;

use crate::defs::{defs, unit_def};
use crate::gadgets::triggers::behavior::{GATE_AREAS, GATE_CLOSED_HEIGHT};
use crate::state::Global;

const AFTER_SPAWN: i32 = 12;

const OPENED_GATES: [i32; 2] = [20, 23];

#[derive(Clone, PartialEq, Eq, Debug)]
struct Census {
    units: usize,
    per_def: Vec<usize>,
}

fn census(spring: &spring::Spring<'_, spring::RulesSynced>) -> Census {
    let units: Vec<i32> = spring.units().get_all_units().unwrap_or_default();
    let defs = defs(spring);
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
            .map(|def| {
                units
                    .iter()
                    .filter(|id| unit_def(spring, **id) == def)
                    .count()
            })
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
                4,
            ),
            first_round: None,
        }
    }
}

impl RestartTracker {
    pub fn is_finished(&self) -> bool {
        self.suite.is_finished()
    }
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut RestartTracker,
    global: &Global,
    frame: i32,
) {
    if t.suite.is_finished() {
        return;
    }
    let spawned = global.world.access().spawn_frame;
    if frame != spawned + AFTER_SPAWN {
        return;
    }
    match t.first_round.clone() {
        None => t.first_round = Some(census(spring)),
        Some(first) => check_restarted(spring, t, global, &first),
    }
}

fn check_restarted(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut RestartTracker,
    global: &Global,
    first: &Census,
) {
    let now = census(spring);
    t.suite.check(spring,
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
    t.suite.check(spring,
        spirit && parts.is_empty() && !game_over && triggered == 0,
        format!(
            "the restarted round starts over: a spirit with no parts, the game running, nothing triggered (spirit {spirit}, parts {parts:?}, over {game_over}, {triggered} triggered)"
        ),
    );
    t.suite.check(spring,
        npc_wisps == 10 && level_units == 61,
        format!(
            "the restarted round tracks the new level: 61 level units and 10 caged NPC wisps (got {level_units} and {npc_wisps})"
        ),
    );
    let heights: Vec<f32> = GATE_AREAS
        .iter()
        .filter(|(key, ..)| OPENED_GATES.contains(key))
        .map(|(_, x1, z1, x2, z2)| {
            spring
                .terrain()
                .get_ground_height((x1 + x2) / 2.0, (z1 + z2) / 2.0)
                .unwrap_or(0.0)
        })
        .collect();
    t.suite.check(spring,
        heights.len() == OPENED_GATES.len()
            && heights
                .iter()
                .all(|height| (height - GATE_CLOSED_HEIGHT).abs() < 1.0),
        format!(
            "the restart closes the gates the first round opened: the doorway terrain is back at {GATE_CLOSED_HEIGHT} (got {heights:?})"
        ),
    );
    t.suite.finish(spring);
}
