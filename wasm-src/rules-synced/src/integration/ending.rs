//! The type-1 ("souls freed") ending scenario, run after the main one.

use alloc::format;
use alloc::string::String;

use springcabal::check::Suite;

use crate::endgame::mark_wisp_dying;
use crate::state::{Endgame, Global};

/// The type-1 ending scenario's progress.
pub struct EndingTracker {
    suite: Suite,
    prepared: bool,
    /// The first ending arrival, where it appeared and on which frame.
    first_arrival: Option<(i32, spring::Float3, i32)>,
    spawn_checked: bool,
}

impl Default for EndingTracker {
    fn default() -> Self {
        Self {
            suite: Suite::new(
                "[PartsNeeded-Rust-Test]",
                "PARTS NEEDED ENDING INTEGRATION TEST",
            ),
            prepared: false,
            first_arrival: None,
            spawn_checked: false,
        }
    }
}

/// How far the first ending arrival must have travelled by frame 260, ~30
/// frames after it appears; the npcwisp unit reaches its 5 elmo/frame top
/// speed after 10 frames.  Its path may lead around a wall, so this checks
/// that it travels, not that every step brings it closer.
const ENDING_MIN_TRAVEL: f32 = 50.0;

fn planar_distance(a: spring::Float3, b: spring::Float3) -> f32 {
    let (dx, dz) = (a.x - b.x, a.z - b.z);
    (dx * dx + dz * dz).sqrt()
}

fn unit_position(id: i32) -> Option<spring::Float3> {
    spring::get_unit_position(id, false, false).ok()
}

/// Whether the unit's current order is game_end.lua's MOVE to `target`, or it
/// already arrived and has no order left.
fn moving_to(id: i32, target: spring::Float3) -> bool {
    match spring::get_unit_commands(id, 1).ok().as_deref() {
        Some([]) => true,
        Some([command, ..]) => {
            command.cmd_id == spring::cmd::MOVE
                && command.params.len() >= 3
                && (command.params[0] - target.x).abs() < 1.0
                && (command.params[2] - target.z).abs() < 1.0
        }
        None => false,
    }
}

/// Exercise the type-1 path separately from the ordinary integration flow.
/// The main test intentionally leaves NPCs alive to cover Ending #1; this
/// second scenario resets only its test-owned state, saves those remaining
/// wisps, and verifies the full authored 500-arrival sequence.
pub fn update(t: &mut EndingTracker, global: &Global, frame: i32) {
    if frame == 180 && !t.prepared {
        prepare(t, global);
    }
    if t.prepared && t.first_arrival.is_none() {
        let first = global.endgame.access().ending_wisps.first().copied();
        if let Some((id, _)) = first
            && let Some(position) = unit_position(id)
        {
            t.first_arrival = Some((id, position, frame));
        }
    }
    if frame == 260 && t.prepared && !t.spawn_checked {
        check_first_arrival(t, global, frame);
        t.spawn_checked = true;
    }
    if frame >= 800
        && t.prepared
        && !t.suite.is_finished()
        && global.endgame.access().sequence_complete
    {
        finish(t, global);
    }
}

/// Reset the ending and save every NPC wisp still alive.
fn prepare(t: &mut EndingTracker, global: &Global) {
    let ids = global.world.access().npc_wisps.clone();
    *global.endgame.access_mut() = Endgame::default();
    for id in ids {
        mark_wisp_dying(global, id, true);
    }
    {
        let mut wisp = global.wisp.access_mut();
        wisp.spirit = false;
        wisp.movement = None;
        wisp.pieces_dirty = true;
    }
    t.prepared = true;
    let _ = spring::echo(
        "[PartsNeeded-Rust-Test] prepared all remaining NPC wisps for type-1 ending",
        "",
    );
}

/// The arrivals spawn 100-400 elmos from the player on each axis, from the
/// synced RNG, so where the first one is by now depends on the seed. Check
/// that it was ordered to the player and is on its way.
fn check_first_arrival(t: &mut EndingTracker, global: &Global, frame: i32) {
    let (game_over, ending_type, release_wisps, target) = {
        let endgame = global.endgame.access();
        (
            endgame.game_over,
            endgame.ending_type,
            endgame.release_wisps,
            endgame.ending_wisps.first().map(|(_, target)| *target),
        )
    };
    t.suite.check(
        game_over && ending_type == 1 && release_wisps,
        "freeing every NPC selects the authored souls-freed ending",
    );
    let progress = t.first_arrival.and_then(|(id, start, seen)| {
        let target = target?;
        let now = unit_position(id)?;
        Some((
            planar_distance(start, target),
            planar_distance(now, target),
            planar_distance(start, now),
            frame - seen,
            moving_to(id, target),
        ))
    });
    t.suite.check(progress
            .is_some_and(|(_, _, travelled, _, ordered)| ordered && travelled >= ENDING_MIN_TRAVEL),
        match progress {
            Some((start, now, travelled, frames, ordered)) => format!(
                "the first ending wisp spawns in the world and heads for the player (moving to the player: {ordered}; {start:.0} -> {now:.0} elmos from it, {travelled:.0} travelled in {frames} frames)"
            ),
            None => String::from(
                "the first ending wisp spawns in the world and heads for the player (not found)",
            ),
        },
    );
}

fn finish(t: &mut EndingTracker, global: &Global) {
    let spawned = global.endgame.access().end_spawned;
    t.suite.check(
        spawned == crate::endgame::ENDING_SPAWN_AMOUNT,
        "the full authored 500-wisp ending sequence completes",
    );
    t.suite.finish();
}
