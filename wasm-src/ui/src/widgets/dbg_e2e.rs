use core::cell::Cell;

use spring_addons::ui::Widget;
use springcabal::shots::{Clock, Screenshots, screenshot};

use crate::state::Global;

/// Frame 77 is after the body-form switch has settled. The rules-side E2E
/// scenario sends movement|1|0 at frame 76, so that screenshot is the visual
/// proof that the physical model faces right while moving right.
const SHOT_FRAMES: [u32; 4] = [30, 77, 86, 100];

/// Without the ending scenario there is nothing left to capture: quit at
/// frame 160.
#[cfg(not(feature = "headless-integration"))]
const QUIT_AFTER_LAST_SHOT: u32 = 60;

/// How long after the type-1 ending starts its final screenshot is taken.
const ENDING_FINAL_SHOT_DELAY: i32 = 180;

fn schedule() -> Screenshots {
    let shots = Screenshots::new(Clock::GameFrame, &SHOT_FRAMES);
    #[cfg(not(feature = "headless-integration"))]
    let shots = shots.quit_after(QUIT_AFTER_LAST_SHOT);
    shots
}

pub struct DbgE2eWidget {
    shots: Screenshots,
    quit_frame: Cell<i32>,
    ending_start: Cell<i32>,
    ending_final_captured: Cell<bool>,
}

impl Default for DbgE2eWidget {
    fn default() -> Self {
        Self {
            shots: schedule(),
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
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let frame = crate::state::current_frame();
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
        if ending.ending_type == 1 && ending.game_over_sequence && self.ending_start.get() < 0 {
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
    }
}
