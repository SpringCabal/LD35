//! The headless integration scenario: the checks and what they measure.
//! `gadgets::integration` drives it.

use springcabal::check::Suite;

use crate::state::{Endgame, Global, Wisp, World};

mod checks;
pub mod ending;
mod probes;
pub mod scenario;

use checks::*;

pub(super) struct Snapshot<'a> {
    pub(super) wisp: &'a Wisp,
    pub(super) world: &'a World,
    pub(super) endgame: &'a Endgame,
}

/// How many checks the main scenario makes.
const CHECKS: u32 = 58;

/// The main scenario's progress and the samples later checks compare with.
pub struct Tracker {
    pub(super) suite: Suite,
    pub(super) initial_npc_wisps: usize,
    /// Whether the UI module's hello reached the rules; the gadget sets it.
    pub ui_hello: bool,
    /// The scenario's commands and movement messages: (sent, received over
    /// RecvLuaMsg); the gadget sets it.
    pub messages: (u32, u32),
    pub(super) wisp_position_at_15: Option<spring::Float3>,
    pub(super) arm_vs_body: Option<f32>,
    pub(super) leg_vs_body: Option<f32>,
    pub(super) lever_piece: Option<f32>,
    pub(super) lever_piece_pulled: Option<f32>,
    pub(super) gate_piece: Option<f32>,
    pub(super) plate_piece: Option<f32>,
    pub(super) arm_swim_changed: bool,
    pub(super) leg_walk_changed: bool,
    /// Prisoners being freed or killed at frame 105, before any bars are open.
    pub(super) dying_before_bars_open: Option<usize>,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            suite: Suite::new("[PartsNeeded-Rust-Test]", "PARTS NEEDED INTEGRATION TESTS")
                .require_at_least(CHECKS),
            initial_npc_wisps: 0,
            ui_hello: false,
            messages: (0, 0),
            wisp_position_at_15: None,
            arm_vs_body: None,
            leg_vs_body: None,
            lever_piece: None,
            lever_piece_pulled: None,
            gate_piece: None,
            plate_piece: None,
            arm_swim_changed: false,
            leg_walk_changed: false,
            dying_before_bars_open: None,
        }
    }
}

/// Run the main scenario's checks due on `frame`.
pub fn update(t: &mut Tracker, global: &Global, frame: i32) {
    if t.suite.is_finished() {
        return;
    }
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
    match frame {
        15 => level_spawned(t, state),
        21 => movement_message(t, state),
        30 => {
            before_pickups(t, state);
            ui_hello(t);
        }
        31 => remote_lever_refused(t, state),
        45 => walking(t, state),
        60 => cages_hold(t, state),
        63 => switch_refused_without_eyes(t, state),
        65 => eyes_collected(t, state),
        71 => lever_at_rest(t, state),
        74 => arm_baseline(t, state),
        75 => body_form_and_lever(t, state),
        77 => {
            lever_pulled(t, state);
            gate_baseline(t, state);
            plate_baseline(t, state);
        }
        78 => form_persists(t, state),
        81 => leg_baseline(t, state),
        84 => lever_released(t, state),
        85 => plate_pressed(t, state),
        91 => {
            swam(t);
            walked(t);
        }
        95 => {
            game_over(t, state);
            plate_sank(t, state);
        }
        105 => after_game_over(t, state),
        120 => {
            one_way_gate(t, state);
            bars_opened_before_release(t, state);
        }
        135 => {
            form_locked_after_game_over(t, state);
            messages_delivered(t);
        }
        140 => {
            gate_opened(t, state);
            plate_stays_down(t, state);
            cage_stays_open(t, state);
        }
        155 => kill_plate(t, state),
        196 => finish(t, state),
        _ => {}
    }
    // Legs arrive at frame 80; from then on the arms follow the walk instead.
    if (75..80).contains(&frame) {
        track_swim(t, state);
    }
    if (82..=90).contains(&frame) {
        track_walk(t, state);
    }
}
