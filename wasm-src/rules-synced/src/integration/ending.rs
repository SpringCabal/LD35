use alloc::format;
use alloc::string::String;

use springcabal::check::Suite;

use partsneeded_shared::{BodyPart, Ending};

use crate::gadgets::endgame::behavior::mark_wisp_dying;
use crate::state::{Endgame, Global, WISP_DESTROY_DELAY};

pub struct EndingTracker {
    suite: Suite,
    prepared: bool,
    first_arrival: Option<(i32, spring::Float3, i32)>,
    spawn_checked: bool,
}

impl Default for EndingTracker {
    fn default() -> Self {
        Self {
            suite: Suite::new(
                "[PartsNeeded-Rust-Test]",
                "PARTS NEEDED ENDING INTEGRATION TEST",
                4,
            ),
            prepared: false,
            first_arrival: None,
            spawn_checked: false,
        }
    }
}

const PREPARE_FRAME: i32 = 220;
const PENDING_CHECK_FRAME: i32 = PREPARE_FRAME + 2;
const SECOND_PICKUP_FRAME: i32 = PREPARE_FRAME + WISP_DESTROY_DELAY + 1;
const ARRIVAL_CHECK_FRAME: i32 = SECOND_PICKUP_FRAME + 34;

const ENDING_MIN_TRAVEL: f32 = 50.0;

fn planar_distance(a: spring::Float3, b: spring::Float3) -> f32 {
    let (dx, dz) = (a.x - b.x, a.z - b.z);
    (dx * dx + dz * dz).sqrt()
}

fn unit_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    id: i32,
) -> Option<spring::Float3> {
    spring
        .units_info()
        .get_unit_position(id, Default::default())
        .ok()
}

fn moving_to(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    id: i32,
    target: spring::Float3,
) -> bool {
    match spring
        .units_commands()
        .get_unit_commands(id, 1)
        .ok()
        .as_deref()
    {
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

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut EndingTracker,
    global: &Global,
    frame: i32,
) {
    if frame == PREPARE_FRAME && !t.prepared {
        prepare(spring, t, global);
    }
    if frame == PENDING_CHECK_FRAME && t.prepared {
        check_pending_rescues(spring, t, global);
    }
    if frame == SECOND_PICKUP_FRAME && t.prepared {
        let saved_wisps = global.endgame.access().saved_wisps;
        *global.endgame.access_mut() = Endgame {
            saved_wisps,
            ..Endgame::default()
        };
        pick_up_last_part_again(spring, global);
    }
    if t.prepared && t.first_arrival.is_none() {
        let first = global.endgame.access().ending_wisps.first().copied();
        if let Some((id, _)) = first
            && let Some(position) = unit_position(spring, id)
        {
            t.first_arrival = Some((id, position, frame));
        }
    }
    if frame == ARRIVAL_CHECK_FRAME && t.prepared && !t.spawn_checked {
        check_first_arrival(spring, t, global, frame);
        t.spawn_checked = true;
    }
    if frame >= 800
        && t.prepared
        && !t.suite.is_finished()
        && global.endgame.access().sequence_complete
    {
        finish(spring, t, global);
    }
}

fn prepare(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut EndingTracker,
    global: &Global,
) {
    let ids = global.world.access().npc_wisps.clone();
    *global.endgame.access_mut() = Endgame::default();
    for id in ids {
        mark_wisp_dying(spring, global, id, true);
    }
    {
        let mut wisp = global.wisp.access_mut();
        wisp.spirit = false;
        wisp.steering.release();
        wisp.pieces_dirty = true;
    }
    pick_up_last_part_again(spring, global);
    t.prepared = true;
    let _ = spring.messages().echo(
        "[PartsNeeded-Rust-Test] saving all remaining NPC wisps for the endings",
        "",
    );
}

fn pick_up_last_part_again(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let (wisp, position) = {
        let mut wisp = global.wisp.access_mut();
        wisp.collected_parts.remove(BodyPart::Mouth);
        (
            wisp.id,
            spring
                .units_info()
                .get_unit_position(wisp.id, Default::default()),
        )
    };
    let def = crate::defs::defs(spring).part_def(BodyPart::Mouth);
    match position {
        Ok(position) => {
            crate::level::spawn_unit(spring, BodyPart::Mouth.name(), def, position, |_, _| {});
        }
        Err(error) => spring::log_warning!(
            spring,
            crate::log::SECTION,
            "the ending scenario can't find the player {wisp}: {error:?}"
        ),
    }
}

fn check_pending_rescues(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut EndingTracker,
    global: &Global,
) {
    let (game_over, ending, pending) = {
        let endgame = global.endgame.access();
        (endgame.game_over, endgame.ending, endgame.dying_wisps.len())
    };
    t.suite.check(spring,
        game_over && ending == Some(Ending::WelcomeBack) && pending == 3,
        format!(
            "the last pickup while 3 rescues are under way ends the game at once, counting them as alive (game over: {game_over}, {ending:?}, {pending} rescues pending)"
        ),
    );
}

fn check_first_arrival(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut EndingTracker,
    global: &Global,
    frame: i32,
) {
    let (game_over, ending, release_wisps, target) = {
        let endgame = global.endgame.access();
        (
            endgame.game_over,
            endgame.ending,
            endgame.release_wisps,
            endgame.ending_wisps.first().map(|(_, target)| *target),
        )
    };
    t.suite.check(
        spring,
        game_over && ending == Some(Ending::SoulsFreed) && release_wisps,
        "the last pickup after every NPC is freed selects the souls-freed ending",
    );
    let progress = t.first_arrival.and_then(|(id, start, seen)| {
        let target = target?;
        let now = unit_position(spring, id)?;
        Some((
            planar_distance(start, target),
            planar_distance(now, target),
            planar_distance(start, now),
            frame - seen,
            moving_to(spring, id, target),
        ))
    });
    t.suite.check(spring, progress
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

fn finish(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut EndingTracker,
    global: &Global,
) {
    let spawned = global.endgame.access().end_spawned;
    t.suite.check(
        spring,
        spawned == crate::gadgets::endgame::behavior::ENDING_SPAWN_AMOUNT,
        "the full authored 500-wisp ending sequence completes",
    );
    t.suite.finish(spring);
}
