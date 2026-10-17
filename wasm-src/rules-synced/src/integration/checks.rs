use alloc::format;

use partsneeded_shared::{BodyPart, Ending};

use super::probes::*;
use super::scenario::{
    KILL_PLATE_CAGES, SCENARIO_MESSAGES, TEST_LEVER, TEST_LEVER_CAGE, TEST_PLATE,
};
use super::{Snapshot, Tracker};
use crate::defs::defs;
use crate::gadgets::triggers::behavior::{LEVER_GATE, level_id};

pub(super) fn level_spawned(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.initial_npc_wisps = state.world.npc_wisps.len();
    t.suite.check(
        spring,
        state.world.level_ids.len() == 61,
        "level static units spawned",
    );
    t.suite
        .check(spring, state.wisp.id >= 0, "player wisp spawned");
    t.suite.check(
        spring,
        state.world.npc_wisps.len() == 10,
        "all ten authored cages spawned their NPC wisps",
    );
    t.suite.check(
        spring,
        worst_wisp_cage_distance(spring, state) < 20.0,
        format!(
            "every caged wisp spawns inside its cage (worst is {:.1} elmos away)",
            worst_wisp_cage_distance(spring, state)
        ),
    );
    t.suite
        .check(spring, state.wisp.spirit, "player starts in spirit form");
    t.suite.check(
        spring,
        state.wisp.body_parts.len() == 6,
        "all six body-part units are tracked",
    );
    t.suite.check(
        spring,
        spring
            .units_pieces()
            .get_unit_piece_map(state.wisp.id)
            .is_ok_and(|pieces| !pieces.is_empty()),
        "wisp model pieces are available to the Rust visibility controller",
    );
    let root_piece = spring
        .units_pieces()
        .get_unit_root_piece(state.wisp.id)
        .ok();
    let root_name = root_piece
        .and_then(|piece| {
            spring
                .units_pieces()
                .get_unit_piece_info(state.wisp.id, piece)
                .ok()
        })
        .map(|info| info.info.name);
    t.suite.check(
        spring,
        root_piece.is_some_and(|piece| piece >= 0),
        format!("wisp model root piece is available ({root_piece:?}, {root_name:?})"),
    );
    t.suite.check(spring, count_units(spring, defs(spring).gate) == 4,
        format!(
            "exactly the four authored gates exist (got {}); the map places these too, so spawning our own duplicates them",
            count_units(spring, defs(spring).gate)
        ),
    );
    for part in BodyPart::ALL {
        let count = count_units(spring, defs(spring).part_def(part));
        t.suite.check(
            spring,
            count == 1,
            format!("exactly one {} pickup exists (got {count})", part.name()),
        );
    }
    t.wisp_position_at_15 = spring
        .units_info()
        .get_unit_position(state.wisp.id, Default::default())
        .ok();
    t.suite.check(
        spring,
        offset_from_body(spring, state.wisp.id, ANIMATED_PIECE).is_some(),
        format!("the wisp model exposes the {ANIMATED_PIECE} piece to query"),
    );
}

pub(super) fn walking(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let now_position = spring
        .units_info()
        .get_unit_position(state.wisp.id, Default::default())
        .ok();
    let heading = spring
        .units_info()
        .get_unit_heading(state.wisp.id, true)
        .ok();
    let displacement = match (t.wisp_position_at_15, now_position) {
        (Some(before), Some(after)) => {
            let dx = after.x - before.x;
            let dz = after.z - before.z;
            (dx * dx + dz * dz).sqrt()
        }
        _ => 0.0,
    };
    t.suite.check(spring, displacement > 0.5,
        format!(
            "WASD movement changes the wisp's world position ({displacement:.2} elmos; start {}, end {}, map {:?})",
            format_position(t.wisp_position_at_15),
            format_position(now_position),
            spring.terrain().get_height_map_size().ok()
        ),
    );
    t.suite.check(spring, heading.is_some_and(|value| (0.5..2.6).contains(&value)),
        format!(
            "the engine turns the wisp toward right while moving right (heading {heading:?} radians)"
        ),
    );
}

pub(super) fn movement_message(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state.wisp.steering.direction() == Some((1.0, 0.0)),
        "movement Lua message reaches wisp controller",
    );
}

pub(super) fn cages_hold(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        worst_wisp_cage_distance(spring, state) < 20.0,
        format!(
            "caged wisps stay in their cages (worst is {:.1} elmos away)",
            worst_wisp_cage_distance(spring, state)
        ),
    );
}

pub(super) fn before_pickups(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        [20, 21, 22, 23]
            .iter()
            .all(|key| state.world.level_ids.contains_key(key)),
        "trigger gate IDs mapped",
    );
    t.suite.check(
        spring,
        !state.endgame.game_over && state.wisp.collected_parts.is_empty(),
        "game remains active before pickups",
    );
    t.suite.check(
        spring,
        state.wisp.steering.direction().is_none(),
        "releasing the movement keys clears the wisp's movement",
    );
}

pub(super) fn remote_lever_refused(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let lever = level_id(state.world, TEST_LEVER);
    t.suite.check(
        spring,
        lever.is_some_and(|id| !state.world.active_triggers.contains(&id)),
        "spirit cannot forge a remote/unarmed lever interaction",
    );
}

pub(super) fn eyes_collected(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state.wisp.collected_parts.contains(BodyPart::Eyes),
        "engine-positioned player collects eyes body part",
    );
    t.suite.check(
        spring,
        !state.endgame.game_over
            && state.wisp.collected_parts.len() == 1
            && state.wisp.body_parts.len() == 5,
        "eyes unlock body form before the remaining parts are collected",
    );
}

pub(super) fn body_form_and_lever(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        !state.wisp.spirit,
        "form-switch Lua message enters body form after eyes are collected",
    );
    let lever = level_id(state.world, TEST_LEVER);
    t.suite.check(
        spring,
        state.wisp.collected_parts.contains(BodyPart::Arms)
            && lever.is_some_and(|id| state.world.active_triggers.contains(&id)),
        "nearby armed body can toggle a lever after form change",
    );
}

pub(super) fn form_persists(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        !state.wisp.spirit,
        "body form persists after the switch instead of bouncing back to spirit",
    );
}

pub(super) fn lever_at_rest(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.lever_piece = lever_piece_offset(spring, state);
    t.suite.check(
        spring,
        t.lever_piece.is_some(),
        "the lever model exposes the Base_001 piece to measure",
    );
}

pub(super) fn lever_pulled(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let now = lever_piece_offset(spring, state);
    let turned = match (t.lever_piece, now) {
        (Some(before), Some(after)) => (before - after).abs() > 0.01,
        _ => false,
    };
    t.suite.check(
        spring,
        turned,
        "pulling the lever runs its unit script: the Base_001 piece turns",
    );
    t.lever_piece_pulled = now;
}

pub(super) fn gate_baseline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.gate_piece = piece_offset(spring, state, LEVER_GATE, GATE_PIECE);
    t.suite.check(
        spring,
        t.gate_piece.is_some(),
        "the gate model exposes the sliding piece to measure",
    );
}

pub(super) fn gate_opened(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let now = piece_offset(spring, state, LEVER_GATE, GATE_PIECE);
    let opened = match (t.gate_piece, now) {
        (Some(before), Some(after)) => (before - after).abs() > 1.0,
        _ => false,
    };
    t.suite.check(
        spring,
        opened,
        "the authored lever combination opens the first gate: its slab slides away",
    );
}

pub(super) fn lever_released(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let lever = level_id(state.world, TEST_LEVER);
    t.suite.check(
        spring,
        lever.is_some_and(|id| !state.world.active_triggers.contains(&id)),
        "a second pull releases the lever instead of latching it on",
    );
    let now = lever_piece_offset(spring, state);
    let returned = match (t.lever_piece, t.lever_piece_pulled, now) {
        (Some(rest), Some(pulled), Some(after)) => {
            (after - pulled).abs() > 0.01 && (after - rest).abs() < 0.01
        }
        _ => false,
    };
    t.suite.check(
        spring,
        returned,
        "releasing the lever runs its unit script again: Base_001 turns back to rest",
    );
}

pub(super) fn arm_baseline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.arm_vs_body = offset_from_body(spring, state.wisp.id, ANIMATED_PIECE);
    t.suite.check(
        spring,
        t.arm_vs_body.is_some(),
        "the wisp model exposes the Body piece to measure the arm against",
    );
}

pub(super) fn track_swim(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let now = offset_from_body(spring, state.wisp.id, ANIMATED_PIECE);
    if let (Some(before), Some(after)) = (t.arm_vs_body, now) {
        t.arm_swim_changed |= (before - after).abs() > 0.01;
    }
}

pub(super) fn leg_baseline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.leg_vs_body = offset_from_body(spring, state.wisp.id, LEG_PIECE);
    t.suite.check(
        spring,
        t.leg_vs_body.is_some(),
        format!("the wisp model exposes the {LEG_PIECE} piece to measure"),
    );
}

pub(super) fn track_walk(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let now = offset_from_body(spring, state.wisp.id, LEG_PIECE);
    if let (Some(before), Some(after)) = (t.leg_vs_body, now) {
        t.leg_walk_changed |= (before - after).abs() > 0.01;
    }
}

pub(super) fn walked(spring: &spring::Spring<'_, spring::RulesSynced>, t: &mut Tracker) {
    t.suite.check(
        spring,
        t.leg_walk_changed,
        format!(
            "the wisp walks: its {LEG_PIECE} animates relative to the body when moving with legs"
        ),
    );
}

pub(super) fn swam(spring: &spring::Spring<'_, spring::RulesSynced>, t: &mut Tracker) {
    t.suite.check(
        spring,
        t.arm_swim_changed,
        "the wisp swims: its arm animates relative to the body when moving with arms but no legs",
    );
}

pub(super) fn plate_pressed(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let plate = level_id(state.world, TEST_PLATE);
    t.suite.check(
        spring,
        state.wisp.collected_parts.contains(BodyPart::Legs)
            && plate.is_some_and(|id| state.world.active_triggers.contains(&id)),
        "embodied player with legs activates an authored standing plate",
    );
}

pub(super) fn game_over(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state.wisp.collected_parts.len() == 6
            && state.wisp.body_parts.is_empty()
            && state.endgame.game_over,
        "all parts trigger the authored game-over state",
    );
    t.suite.check(
        spring,
        state.endgame.ending == Some(Ending::WelcomeBack),
        "living wisps select the authored Welcome back ending",
    );
    t.suite.check(
        spring,
        state.endgame.sequence_complete,
        "non-animated endings immediately expose the ending screen",
    );
}

pub(super) fn form_locked_after_game_over(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        !state.wisp.spirit,
        "game-over blocks further player form-switch input",
    );
}

pub(super) fn before_bars_open(t: &mut Tracker, state: &Snapshot<'_>) {
    t.dying_before_bars_open = Some(state.endgame.dying_wisps.len());
}

pub(super) fn pickups_removed(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state
            .world
            .level_ids
            .iter()
            .filter(|(key, _)| [27468, 27489, 22389, 4736, 18744, 30249].contains(*key))
            .all(|(_, id)| spring.units_info().get_unit_def_id(*id).is_err()),
        "collected pickup units are removed without stale render models",
    );
}

pub(super) fn one_way_gate(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let gate = level_id(state.world, 23);
    t.suite.check(
        spring,
        gate.is_some_and(|id| state.world.active_triggerables.contains(&id)),
        "the last-room standing plate leaves gate 23 open after the player steps off it",
    );
}

pub(super) fn finish(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state.endgame.saved_wisps == 3,
        "lever plus standing plate open all three linked cages and complete delayed rescues",
    );
    t.suite.check(
        spring,
        state.endgame.killed_wisps == KILL_PLATE_CAGES as i32,
        format!(
            "the kill plate's {KILL_PLATE_CAGES} prisoners count as killed (got {})",
            state.endgame.killed_wisps
        ),
    );
    let gone = 3 + KILL_PLATE_CAGES;
    t.suite.check(spring,
        state.world.npc_wisps.len() + gone == t.initial_npc_wisps,
        format!(
            "expired rescued and killed wisps leave the authoritative alive-wisp set (expected {}, got {})",
            t.initial_npc_wisps.saturating_sub(gone),
            state.world.npc_wisps.len()
        ),
    );
    t.suite.finish(spring);
}

pub(super) fn messages_delivered(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
) {
    let (sent, received) = t.messages;
    t.suite.check(spring,
        sent == SCENARIO_MESSAGES && received == sent,
        format!(
            "the scenario's commands and movement reach the rules over RecvLuaMsg ({received} of {sent} sent, {SCENARIO_MESSAGES} expected)"
        ),
    );
}

pub(super) fn switch_refused_without_eyes(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.suite.check(
        spring,
        state.wisp.spirit && state.wisp.form_frame == 0,
        "without eyes a form switch is refused and starts no cooldown",
    );
}

pub(super) fn plate_baseline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    t.plate_piece = piece_offset(spring, state, TEST_PLATE, PLATE_PIECE);
    t.suite.check(
        spring,
        t.plate_piece.is_some(),
        format!("the plate model exposes the {PLATE_PIECE} piece to measure"),
    );
}

fn plate_sunk(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &Tracker,
    state: &Snapshot<'_>,
) -> Option<f32> {
    Some((t.plate_piece? - piece_offset(spring, state, TEST_PLATE, PLATE_PIECE)?).abs())
}

pub(super) fn plate_sank(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let sunk = plate_sunk(spring, t, state);
    t.suite.check(
        spring,
        sunk.is_some_and(|sunk| sunk > 1.0),
        format!("pressing a plate runs its unit script: the {PLATE_PIECE} sinks ({sunk:?} elmos)"),
    );
}

pub(super) fn plate_stays_down(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let sunk = plate_sunk(spring, t, state);
    t.suite.check(
        spring,
        sunk.is_some_and(|sunk| sunk > 1.0)
            && level_id(state.world, TEST_PLATE)
                .is_some_and(|id| state.world.active_triggers.contains(&id)),
        format!("a plate stays pressed after the player steps off ({sunk:?} elmos)"),
    );
}

pub(super) fn bars_opened_before_release(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let before = t.dying_before_bars_open;
    let now = state.endgame.dying_wisps.len();
    t.suite.check(spring,
        before == Some(0) && now == 1,
        format!(
            "a cage frees its prisoner once its bars are open (dying at frame 105: {before:?}, at 120: {now})"
        ),
    );
}

pub(super) fn kill_plate(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let killed = state
        .endgame
        .dying_wisps
        .iter()
        .filter(|(_, saved, _)| !saved)
        .count();
    t.suite.check(spring,
        killed == KILL_PLATE_CAGES,
        format!("a kill plate kills the prisoners of its {KILL_PLATE_CAGES} cages at once (got {killed})"),
    );
}

pub(super) fn cage_stays_open(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    t: &mut Tracker,
    state: &Snapshot<'_>,
) {
    let cage = level_id(state.world, TEST_LEVER_CAGE);
    t.suite.check(
        spring,
        cage.is_some_and(|id| state.world.active_triggerables.contains(&id)),
        "a cage stays open after the lever that opened it is released",
    );
}
