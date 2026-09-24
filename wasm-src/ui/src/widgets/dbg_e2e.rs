use core::cell::Cell;

use partsneeded_shared::Ending;
use spring_addons::ui::Widget;
use springcabal::shots::{Clock, Screenshots, screenshot};

use crate::state::Global;

/// Shot 2 is the start room once the switch to body form (frame 66 in the
/// rules' scenario) has settled. The screen's spirit-to-body fade runs 15
/// frames from the update in which the UI hears of the switch, usually frame
/// 67, so it is over by frame 84 unless the UI misses more than two frames. A
/// shot inside the fade catches a different fade state whenever the capture
/// lands a frame or two later, which happens whenever the UI's updates skip
/// frames. The body itself is out of view by 84: the scenario moves it to a
/// plate in another room at frame 82. Shot 2 shows the body-form view and the
/// lever the body pulled twice, back at rest.
///
const SHOT_FRAMES: [u32; 2] = [30, 84];

/// Shots 3 and 4 show the body standing on the plate the scenario moved it
/// to at frame 82 (it stops at 91): with eyes, arms and legs, then with two
/// parts more (picked up at 124 and 126). The camera follows the wisp with
/// wall-clock smoothing, so where it is on a given frame depends on how long
/// the earlier captures took; each shot waits for it to settle instead, from
/// its first frame until the frame before the scenario changes the scene.
const SETTLED_SHOTS: [SettledShot; 2] = [
    SettledShot {
        label: "3 (body on the plate)",
        from: 112,
        until: 124,
    },
    SettledShot {
        label: "4 (five parts)",
        from: 130,
        until: 138,
    },
];

/// The camera has settled when it moved less than this (elmos) in each of
/// `SETTLED_UPDATES` updates in a row.
const SETTLED_STEP: f32 = 0.5;
const SETTLED_UPDATES: u32 = 3;

/// Without the ending scenario there is nothing left to capture: quit at
/// frame 198.
#[cfg(not(feature = "headless-integration"))]
const QUIT_AFTER_LAST_SHOT: i32 = 60;

struct SettledShot {
    label: &'static str,
    from: i32,
    /// Taken on this frame even if the camera is still moving, and logged
    /// as unsettled: the suite fails on that line.
    until: i32,
}

/// How long after the type-1 ending starts its final screenshot is taken.
const ENDING_FINAL_SHOT_DELAY: i32 = 180;

pub struct DbgE2eWidget {
    shots: Screenshots,
    settled_taken: Cell<usize>,
    last_camera: Cell<Option<spring::Float3>>,
    still_updates: Cell<u32>,
    quit_frame: Cell<i32>,
    ending_start: Cell<i32>,
    ending_final_captured: Cell<bool>,
}

impl Default for DbgE2eWidget {
    fn default() -> Self {
        Self {
            shots: Screenshots::new(Clock::GameFrame, &SHOT_FRAMES),
            settled_taken: Cell::new(0),
            last_camera: Cell::new(None),
            still_updates: Cell::new(0),
            quit_frame: Cell::new(-1),
            ending_start: Cell::new(-1),
            ending_final_captured: Cell::new(false),
        }
    }
}

impl DbgE2eWidget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates in a row the camera has barely moved.
    fn track_camera(&self) -> u32 {
        let Ok(now) = spring::camera::get_camera_position(0) else {
            self.still_updates.set(0);
            return 0;
        };
        let still = self.last_camera.replace(Some(now)).is_some_and(|last| {
            let (dx, dy, dz) = (now.x - last.x, now.y - last.y, now.z - last.z);
            dx * dx + dy * dy + dz * dz < SETTLED_STEP * SETTLED_STEP
        });
        let updates = if still {
            self.still_updates.get() + 1
        } else {
            0
        };
        self.still_updates.set(updates);
        updates
    }

    /// Take the next settled shot once its camera is still.
    fn settled_shot(&self, frame: i32) {
        let still_updates = self.track_camera();
        let Some(shot) = SETTLED_SHOTS.get(self.settled_taken.get()) else {
            return;
        };
        if frame < shot.from {
            return;
        }
        let settled = still_updates >= SETTLED_UPDATES;
        if !settled && frame < shot.until {
            return;
        }
        if !settled {
            spring::log_warning!(
                "partsneeded",
                "E2E camera not settled for screenshot {}",
                shot.label
            );
        }
        screenshot(&alloc::format!("{} at {frame}", shot.label));
        self.settled_taken.set(self.settled_taken.get() + 1);
        #[cfg(not(feature = "headless-integration"))]
        if self.settled_taken.get() == SETTLED_SHOTS.len() {
            self.quit_frame.set(frame + QUIT_AFTER_LAST_SHOT);
        }
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let frame = spring::game::current_frame();
        let quit_frame = self.quit_frame.get();
        if quit_frame >= 0 && frame >= quit_frame {
            let _ = spring::system_control::quit(0);
            return;
        }
        // The normal smoke screenshots finish before the ceremony starts.
        // In the Parts Needed E2E build the synced integration scenario also
        // exercises Ending #2; capture its completed visual sequence before
        // ending the run so the converging wisps and final overlay are tested,
        // not just the rules-side marker.
        let ending = ctx.global().player.access();
        if ending.ending == Some(Ending::SoulsFreed)
            && ending.game_over_sequence
            && self.ending_start.get() < 0
        {
            self.ending_start.set(frame);
            screenshot("ending start");
            return;
        }
        drop(ending);
        let ending_start = self.ending_start.get();
        if ending_start >= 0
            && frame - ending_start >= ENDING_FINAL_SHOT_DELAY
            && !self.ending_final_captured.replace(true)
        {
            screenshot("ending final");
            self.quit_frame.set(frame + 30);
            return;
        }
        self.shots.update(ctx, dt);
        self.settled_shot(frame);
    }
}
