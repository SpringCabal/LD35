use alloc::format;
use core::cell::RefCell;

use partsneeded_shared::BodyPart;

use crate::endgame::{game_over_type, mark_wisp_dying};
use crate::state::{Endgame, Global, Wisp, World, param};
use crate::triggers::{LEVER_GATE, level_id};

/// The lever that opens cage 23261; the scenario pulls it twice.
pub const TEST_LEVER: i32 = 14725;
/// A standing plate linked to two cages; the embodied player with legs stands
/// on it.
pub const TEST_PLATE: i32 = 12701;
/// The last room's one-way plate, which opens gate 23.
pub const LAST_ROOM_PLATE: i32 = 11688;

/// How many live units of a given unitdef exist right now.
fn count_units_named(name: &str) -> usize {
    let Ok(def) = spring::get_unit_def_id_by_name(name) else {
        return usize::MAX;
    };
    let Ok(units) = spring::get_all_units() else {
        return usize::MAX;
    };
    units
        .into_iter()
        .filter(|id| spring::get_unit_def_id(*id).ok() == Some(spring::DefId(def)))
        .count()
}

/// A child piece Scripts/wisp.lua actually moves when the upper arm turns.
/// Measuring the ArmR pivot itself is unreliable: rotating a piece does not
/// move that piece's origin, while its forearm child visibly follows it.
const ANIMATED_PIECE: &str = "ForeArmR";

/// The lever arm's facing.
///
/// Scripts/lever.lua Turns the piece about its own pivot, which leaves its
/// position unchanged -- only the direction moves, so that is what to sample.
fn lever_piece_offset(state: &Snapshot<'_>) -> Option<f32> {
    let lever = level_id(state.world, TEST_LEVER)?;
    let piece_number = spring::get_unit_piece_map(lever)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "Base_001")
        .map(|piece| piece.piece_num)?;
    let piece = spring::owned::units_pieces::get_unit_piece_pos_dir(lever, piece_number).ok()?;
    Some(piece.direction.x + piece.direction.y * 2.0 + piece.direction.z * 3.0)
}

/// Distance from the animated arm piece to the body piece it hangs off.
///
/// Float_Body bobs the whole model continuously, so measuring the arm against
/// the unit centre cannot tell a swim stroke from that idle bob.  The arm
/// animations rotate the arm relative to Body, so this only moves when the arm
/// itself is animating.
fn arm_offset_from_body(unit_id: i32) -> Option<f32> {
    let arm = spring::typed::unit_piece_position_by_name(unit_id, ANIMATED_PIECE)?;
    let body = spring::typed::unit_piece_position_by_name(unit_id, "Body")?;
    let (dx, dy, dz) = (arm.x - body.x, arm.y - body.y, arm.z - body.z);
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

/// Distance from the unit's own centre to its animated piece.
///
/// The piece query reports a world position, which moves whenever the unit
/// walks -- comparing that directly passed even with the unit scripts disabled.
/// Measuring against the unit's own position is invariant to both translation
/// and heading, so it only changes when the piece really is animating.
fn piece_offset_from_unit(unit_id: i32) -> Option<f32> {
    let piece = spring::typed::unit_piece_position_by_name(unit_id, ANIMATED_PIECE)?;
    let unit = spring::get_unit_position(unit_id, false, false).ok()?;
    let (dx, dy, dz) = (piece.x - unit.x, piece.y - unit.y, piece.z - unit.z);
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

/// The furthest any NPC wisp sits from the cage nearest it.
///
/// unit_cage.lua spawns each one at its cage's exact position, so they should
/// all read essentially zero; a wisp loose on the floor is a prisoner that
/// escaped its cell before the player ever arrived.
fn worst_wisp_cage_distance(state: &Snapshot<'_>) -> f32 {
    let Ok(cage_def) = spring::get_unit_def_id_by_name("cage") else {
        return f32::MAX;
    };
    let Ok(units) = spring::get_all_units() else {
        return f32::MAX;
    };
    let cages: alloc::vec::Vec<spring::Float3> = units
        .into_iter()
        .filter(|id| spring::get_unit_def_id(*id).ok() == Some(spring::DefId(cage_def)))
        .filter_map(|id| spring::get_unit_position(id, false, false).ok())
        .collect();
    if cages.is_empty() {
        return f32::MAX;
    }
    let mut worst = 0.0f32;
    for wisp in &state.world.npc_wisps {
        let Ok(position) = spring::get_unit_position(*wisp, false, false) else {
            continue;
        };
        let nearest = cages
            .iter()
            .map(|cage| {
                let (dx, dz) = (cage.x - position.x, cage.z - position.z);
                dx * dx + dz * dz
            })
            .fold(f32::MAX, f32::min)
            .sqrt();
        worst = worst.max(nearest);
    }
    worst
}

/// How far the gate's moving slab sits from the gate unit's own origin.
///
/// Scripts/gate.lua slides `Cube41_extract45_GATE` 168 elmos along z when the
/// bitmask link opens the door.  Measuring the piece is what proves the door
/// actually opened: the Rust-side trigger bookkeeping can say "open" while the
/// unit script never ran.
fn gate_piece_offset(state: &Snapshot<'_>, key: i32) -> Option<f32> {
    let gate = level_id(state.world, key)?;
    let piece_number = spring::get_unit_piece_map(gate)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "Cube41_extract45_GATE")
        .map(|piece| piece.piece_num)?;
    let piece = spring::owned::units_pieces::get_unit_piece_pos_dir(gate, piece_number).ok()?;
    let unit = spring::get_unit_position(gate, false, false).ok()?;
    let (dx, dy, dz) = (
        piece.position.x - unit.x,
        piece.position.y - unit.y,
        piece.position.z - unit.z,
    );
    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

struct Snapshot<'a> {
    wisp: &'a Wisp,
    world: &'a World,
    endgame: &'a Endgame,
}

#[derive(Default)]
struct Tracker {
    passed: u32,
    failed: u32,
    done: bool,
    initial_npc_wisps: usize,
    wisp_position_at_15: Option<spring::Float3>,
    arms_at_15: Option<f32>,
    arm_vs_body: Option<f32>,
    lever_piece: Option<f32>,
    lever_piece_pulled: Option<f32>,
    gate_piece: Option<f32>,
    arm_swim_changed: bool,
}
thread_local! { static TRACKER: RefCell<Tracker> = RefCell::new(Tracker::default()); }
fn check(t: &mut Tracker, condition: bool, message: &str) {
    if condition {
        t.passed += 1;
        let _ = spring::echo(
            &format!(
                "[PartsNeeded-Rust-Test] ASSERTION PASSED ({}): {message}",
                t.passed
            ),
            "",
        );
    } else {
        t.failed += 1;
        let _ = spring::echo(
            &format!("[PartsNeeded-Rust-Test] ASSERTION FAILED: {message}"),
            "",
        );
    }
}
pub fn update(global: &Global, frame: i32) {
    // Test-only bookkeeping; nothing below issues a callout that re-enters a
    // callin wanting these resources.
    let wisp = global.wisp.access();
    let world = global.world.access();
    let endgame = global.endgame.access();
    let state = &Snapshot {
        wisp: &wisp,
        world: &world,
        endgame: &endgame,
    };
    TRACKER.with_borrow_mut(|t| {
        if t.done {
            return;
        }
        if frame == 15 {
            t.initial_npc_wisps = state.world.npc_wisps.len();
            check(
                t,
                [
                    game_over_type(1, 0, 0) == 0,
                    game_over_type(0, 1, 0) == 1,
                    game_over_type(0, 0, 1) == 2,
                    game_over_type(0, 0, 0) == 3,
                ]
                .into_iter()
                .all(|branch| branch),
                "all four authored game-over outcome branches select correctly",
            );
            check(t, state.world.level_ids.len() == 61, "level static units spawned");
            check(t, state.wisp.id >= 0, "player wisp spawned");
            check(
                t,
                state.world.npc_wisps.len() == 10,
                "all ten authored cages spawned their NPC wisps",
            );
            check(
                t,
                worst_wisp_cage_distance(state) < 20.0,
                &format!(
                    "every caged wisp spawns inside its cage (worst is {:.1} elmos away)",
                    worst_wisp_cage_distance(state)
                ),
            );
            check(t, state.wisp.spirit, "player starts in spirit form");
            check(
                t,
                state.wisp.body_parts.len() == 6,
                "all six body-part units are tracked",
            );
            check(
                t,
                spring::get_unit_piece_map(state.wisp.id).is_ok_and(|pieces| !pieces.is_empty()),
                "wisp model pieces are available to the Rust visibility controller",
            );
            let root_piece = spring::get_unit_root_piece(state.wisp.id).ok();
            let root_name = root_piece
                .and_then(|piece| spring::get_unit_piece_info(state.wisp.id, piece).ok())
                .map(|info| info.info.name);
            check(
                t,
                root_piece.is_some_and(|piece| piece >= 0),
                &format!("wisp model root piece is available ({root_piece:?}, {root_name:?})"),
            );
            check(
                t,
                count_units_named("gatesmoth") == 4,
                &format!(
                    "exactly the four authored gates exist (got {}); the map places these too, so spawning our own duplicates them",
                    count_units_named("gatesmoth")
                ),
            );
            for part in BodyPart::ALL.map(BodyPart::name) {
                check(
                    t,
                    count_units_named(part) == 1,
                    &format!(
                        "exactly one {part} pickup exists (got {})",
                        count_units_named(part)
                    ),
                );
            }
            t.wisp_position_at_15 = spring::get_unit_position(state.wisp.id, false, false).ok();
            t.arms_at_15 = piece_offset_from_unit(state.wisp.id);
            check(
                t,
                t.arms_at_15.is_some(),
                &format!("the wisp model exposes the {ANIMATED_PIECE} piece to query"),
            );
        }
        if frame == 45 {
            let now_position = spring::get_unit_position(state.wisp.id, false, false).ok();
            let heading = spring::rules_synced::units_info::get_unit_heading(state.wisp.id, true).ok();
            let displacement = match (t.wisp_position_at_15, now_position) {
                (Some(before), Some(after)) => {
                    let dx = after.x - before.x;
                    let dz = after.z - before.z;
                    (dx * dx + dz * dz).sqrt()
                }
                _ => 0.0,
            };
            check(
                t,
                displacement > 0.5,
                &format!(
                    "WASD movement changes the wisp's world position ({displacement:.2} elmos; start {}, end {}, map {:?})",
                    format_position(t.wisp_position_at_15),
                    format_position(now_position),
                    spring::get_height_map_size().ok()
                ),
            );
            check(
                t,
                heading.is_some_and(|value| (0.5..2.6).contains(&value)),
                &format!(
                    "the engine turns the wisp toward right while moving right (heading {heading:?} radians)"
                ),
            );
            // The wisp has been walking since frame 20.  Scripts/wisp.lua drives
            // Walk()/Wave_Arms() from StartMoving, so the arms piece has to have
            // moved relative to the model; if the unit scripts are not running
            // it sits at its authored rest position forever.
            let now = piece_offset_from_unit(state.wisp.id);
            let moved = match (t.arms_at_15, now) {
                (Some(before), Some(after)) => (before - after).abs() > 0.01,
                _ => false,
            };
            check(
                t,
                moved,
                &format!("the wisp is animated: its {ANIMATED_PIECE} piece moves while walking"),
            );
        }
        if frame == 21 {
            check(
                t,
                state.wisp.movement == Some((1.0, 0.0)),
                "movement Lua message reaches wisp controller",
            );
        }
        if frame == 60 {
            // And they have to still be there: nothing should be nudging a
            // prisoner out through the bars while the player is elsewhere.
            check(
                t,
                worst_wisp_cage_distance(state) < 20.0,
                &format!(
                    "caged wisps stay in their cages (worst is {:.1} elmos away)",
                    worst_wisp_cage_distance(state)
                ),
            );
        }
        if frame == 30 {
            check(
                t,
                [20, 21, 22, 23].iter().all(|key| {
                    state
                        .world
                        .level_ids
                        .iter()
                        .any(|(candidate, _)| candidate == key)
                }),
                "trigger gate IDs mapped",
            );
            check(
                t,
                !state.endgame.game_over && state.wisp.collected_parts.is_empty(),
                "game remains active before pickups",
            );
            check(
                t,
                state.wisp.movement.is_none(),
                "movement input expires without fresh UI samples",
            );
        }
        if frame == 31 {
            let lever = level_id(state.world, TEST_LEVER);
            check(
                t,
                lever.is_some_and(|id| !state.world.active_triggers.contains(&id)),
                "spirit cannot forge a remote/unarmed lever interaction",
            );
        }
        if frame == 65 {
            check(
                t,
                state.wisp.collected_parts.contains(BodyPart::Eyes),
                "engine-positioned player collects eyes body part",
            );
            check(
                t,
                !state.endgame.game_over && state.wisp.collected_parts.len() == 1 && state.wisp.body_parts.len() == 5,
                "eyes unlock body form before the remaining parts are collected",
            );
        }
        if frame == 75 {
            check(
                t,
                !state.wisp.spirit,
                "form-switch Lua message enters body form after eyes are collected",
            );
            let lever = level_id(state.world, TEST_LEVER);
            check(
                t,
                state.wisp.collected_parts.contains(BodyPart::Arms)
                    && lever.is_some_and(|id| state.world.active_triggers.contains(&id)),
                "nearby armed body can toggle a lever after form change",
            );
        }
        if frame == 78 {
            // The form must stay switched: the UI's prediction must not race
            // the status snapshot back into spirit form.
            check(
                t,
                !state.wisp.spirit,
                "body form persists after the switch instead of bouncing back to spirit",
            );
        }
        if frame == 71 {
            // Before the lever is pulled.  Scripts/lever.lua turns Base_001 in
            // its SetWasmActivated, so the piece moving is proof the unit script
            // actually ran -- the Rust-side trigger state alone is not.
            t.lever_piece = lever_piece_offset(state);
            check(
                t,
                t.lever_piece.is_some(),
                "the lever model exposes the Base_001 piece to measure",
            );
        }
        if frame == 77 {
            let now = lever_piece_offset(state);
            let turned = match (t.lever_piece, now) {
                (Some(before), Some(after)) => (before - after).abs() > 0.01,
                _ => false,
            };
            check(
                t,
                turned,
                "pulling the lever runs its unit script: the Base_001 piece turns",
            );
            t.lever_piece_pulled = now;
        }
        if frame == 77 {
            // Sampled before the combination is set, as the baseline for the
            // frame-140 check.
            t.gate_piece = gate_piece_offset(state, LEVER_GATE);
            check(
                t,
                t.gate_piece.is_some(),
                "the gate model exposes the sliding piece to measure",
            );
        }
        if frame == 140 {
            // gate.lua sleeps half a second and then slides the slab, so give
            // the door time to finish opening before measuring it.
            //
            // Reading the negative ids in game_custom_spawn.lua as "must be
            // off" inverts all four levers and leaves this door unopenable.
            let now = gate_piece_offset(state, LEVER_GATE);
            let opened = match (t.gate_piece, now) {
                (Some(before), Some(after)) => (before - after).abs() > 1.0,
                _ => false,
            };
            check(
                t,
                opened,
                "the authored lever combination opens the first gate: its slab slides away",
            );
        }
        if frame == 84 {
            // Levers are toggles in ui_wisp_control.lua / unit_wisp_controller.lua:
            // a second click flips them back off.  A one-shot lever would strand
            // the player behind any door they opened by mistake.
            let lever = level_id(state.world, TEST_LEVER);
            check(
                t,
                lever.is_some_and(|id| !state.world.active_triggers.contains(&id)),
                "a second pull releases the lever instead of latching it on",
            );
            let now = lever_piece_offset(state);
            let returned = match (t.lever_piece, t.lever_piece_pulled, now) {
                (Some(rest), Some(pulled), Some(after)) => {
                    (after - pulled).abs() > 0.01 && (after - rest).abs() < 0.01
                }
                _ => false,
            };
            check(
                t,
                returned,
                "releasing the lever runs its unit script again: Base_001 turns back to rest",
            );
        }
        if frame == 74 {
            t.arm_vs_body = arm_offset_from_body(state.wisp.id);
            check(
                t,
                t.arm_vs_body.is_some(),
                "the wisp model exposes the Body piece to measure the arm against",
            );
        }
        if (75..=90).contains(&frame) {
            let now = arm_offset_from_body(state.wisp.id);
            if let (Some(before), Some(after)) = (t.arm_vs_body, now) {
                t.arm_swim_changed |= (before - after).abs() > 0.01;
            }
        }
        if frame == 91 {
            check(
                t,
                t.arm_swim_changed,
                "the wisp swims: its arm animates relative to the body when moving with arms but no legs",
            );
        }
        if frame == 85 {
            let plate = level_id(state.world, TEST_PLATE);
            check(
                t,
                state.wisp.collected_parts.contains(BodyPart::Legs)
                    && plate.is_some_and(|id| state.world.active_triggers.contains(&id)),
                "embodied player with legs activates an authored standing plate",
            );
        }
        if frame == 95 {
            check(
                t,
                state.wisp.collected_parts.len() == 6 && state.wisp.body_parts.is_empty() && state.endgame.game_over,
                "all parts trigger the authored game-over state",
            );
            check(
                t,
                state.endgame.ending_type == 0,
                "living wisps select the authored Welcome back ending",
            );
            check(
                t,
                state.endgame.sequence_complete,
                "non-animated endings immediately expose the ending screen",
            );
        }
        if frame == 105 {
            check(
                t,
                !state.wisp.spirit,
                "game-over blocks further player form-switch input",
            );
            check(
                t,
                state
                    .world
                    .level_ids
                    .iter()
                    .filter(|(key, _)| [27468, 27489, 22389, 4736, 18744, 30249].contains(key))
                    .all(|(_, id)| spring::get_unit_def_id(*id).is_err()),
                "collected pickup units are removed without stale render models",
            );
        }
        if frame == 120 {
            let gate = level_id(state.world, 23);
            check(
                t,
                gate.is_some_and(|id| state.world.active_triggerables.contains(&id)),
                "the last-room standing plate leaves gate 23 open after the player steps off it",
            );
        }
        if frame == 145 {
            check(
                t,
                state.endgame.saved_wisps == 3,
                "lever plus standing plate open all three linked cages and complete delayed rescues",
            );
            check(
                t,
                state.world.npc_wisps.len() + 3 == t.initial_npc_wisps,
                &format!(
                    "expired rescued wisp leaves the authoritative alive-wisp set (expected {}, got {})",
                    t.initial_npc_wisps.saturating_sub(3),
                    state.world.npc_wisps.len()
                ),
            );
            let _ = spring::echo(
                if t.failed == 0 {
                    "[PartsNeeded-Rust-Test] ALL PARTS NEEDED INTEGRATION TESTS PASSED"
                } else {
                    "[PartsNeeded-Rust-Test] PARTS NEEDED INTEGRATION TESTS FAILED"
                },
                "",
            );
            t.done = true;
        }
    });
}

fn format_position(position: Option<spring::Float3>) -> alloc::string::String {
    match position {
        Some(position) => alloc::format!("({:.1}, {:.1})", position.x, position.z),
        None => alloc::string::String::from("unavailable"),
    }
}

#[derive(Default)]
struct EndingTracker {
    prepared: bool,
    spawn_checked: bool,
    done: bool,
    failed: u32,
}

thread_local! { static ENDING_TRACKER: RefCell<EndingTracker> = RefCell::new(EndingTracker::default()); }

fn ending_check(t: &mut EndingTracker, condition: bool, message: &str) {
    if condition {
        let _ = spring::echo(
            &format!("[PartsNeeded-Rust-Test] ENDING ASSERTION PASSED: {message}"),
            "",
        );
    } else {
        t.failed += 1;
        let _ = spring::echo(
            &format!("[PartsNeeded-Rust-Test] ENDING ASSERTION FAILED: {message}"),
            "",
        );
    }
}

/// Exercise the type-1 path separately from the ordinary integration flow.
/// The main test intentionally leaves NPCs alive to cover Ending #1; this
/// second scenario resets only its test-owned state, saves those remaining
/// wisps, and verifies the full authored 500-arrival sequence.
pub fn update_ending_integration(global: &Global, frame: i32) {
    ENDING_TRACKER.with_borrow_mut(|t| {
        if frame == 180 && !t.prepared {
            let ids = global.world.access().npc_wisps.clone();
            {
                let mut endgame = global.endgame.access_mut();
                endgame.game_over = false;
                endgame.ending_type = 0;
                endgame.dying_wisps.clear();
                endgame.saved_wisps = 0;
                endgame.killed_wisps = 0;
                endgame.release_wisps = false;
                endgame.end_spawned = 0;
                endgame.end_spawn_frame = 0;
                endgame.ending_wisps.clear();
                endgame.sequence_complete = false;
            }
            for id in ids {
                mark_wisp_dying(global, id, true);
            }
            {
                let mut wisp = global.wisp.access_mut();
                wisp.spirit = false;
                wisp.movement = None;
                wisp.pieces_dirty = true;
            }
            param("game_over", 0.0);
            param("game_over_sequence", 0.0);
            param("game_over_type", 0.0);
            param("spiritMode", 0.0);
            t.prepared = true;
            let _ = spring::echo(
                "[PartsNeeded-Rust-Test] prepared all remaining NPC wisps for type-1 ending",
                "",
            );
        }

        // Give the first normal engine MOVE order a few frames to accelerate
        // before checking its convergence. The Lua unit has maxVelocity=5 and
        // acceleration=0.5, so checking immediately after creation made the
        // test depend on the first random offset rather than movement.
        if frame == 260 && t.prepared && !t.spawn_checked {
            let (game_over, ending_type, release_wisps, first_ending) = {
                let endgame = global.endgame.access();
                (
                    endgame.game_over,
                    endgame.ending_type,
                    endgame.release_wisps,
                    endgame.ending_wisps.first().copied(),
                )
            };
            ending_check(
                t,
                game_over && ending_type == 1 && release_wisps,
                "freeing every NPC selects the authored souls-freed ending",
            );
            let converging = first_ending.is_some_and(|(id, target)| {
                spring::get_unit_position(id, false, false)
                    .ok()
                    .is_some_and(|position| {
                        let dx = position.x - target.x;
                        let dz = position.z - target.z;
                        (dx * dx + dz * dz).sqrt() < 300.0
                    })
            });
            ending_check(
                t,
                converging,
                "the first ending wisp spawns in the world and converges toward the player",
            );
            t.spawn_checked = true;
        }

        if frame >= 800 && t.prepared && !t.done {
            let (complete, spawned) = {
                let endgame = global.endgame.access();
                (endgame.sequence_complete, endgame.end_spawned)
            };
            if complete {
                ending_check(
                    t,
                    spawned == crate::endgame::ENDING_SPAWN_AMOUNT,
                    "the full authored 500-wisp ending sequence completes",
                );
                let _ = spring::echo(
                    if t.failed == 0 {
                        "[PartsNeeded-Rust-Test] PARTS NEEDED ENDING INTEGRATION TEST PASSED"
                    } else {
                        "[PartsNeeded-Rust-Test] PARTS NEEDED ENDING INTEGRATION TEST FAILED"
                    },
                    "",
                );
                t.done = true;
            }
        }
    });
}
