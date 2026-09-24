use spring_addons::ui::Widget;

use crate::state::Global;

pub struct DbgE2eWidget {
    captured: core::cell::Cell<u8>,
    quit_frame: core::cell::Cell<i32>,
    ending_start: core::cell::Cell<i32>,
    ending_final_captured: core::cell::Cell<bool>,
}

impl Default for DbgE2eWidget {
    fn default() -> Self {
        Self {
            captured: core::cell::Cell::new(0),
            quit_frame: core::cell::Cell::new(-1),
            ending_start: core::cell::Cell::new(-1),
            ending_final_captured: core::cell::Cell::new(false),
        }
    }
}

impl DbgE2eWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

fn screenshot() {
    if let Err(error) = spring::send_commands("screenshot", "") {
        spring::log_warning!("partsneeded", "screenshot failed: {error:?}");
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
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
            screenshot();
            return;
        }
        let ending_start = self.ending_start.get();
        if ending_start >= 0
            && frame - ending_start >= 180
            && !self.ending_final_captured.replace(true)
        {
            screenshot();
            self.quit_frame.set(frame + 30);
            return;
        }
        let target = self.captured.get();
        // Frame 77 is after the body-form switch has settled. The rules-side
        // E2E scenario sends movement|1|0 at frame 76, so this screenshot is
        // the visual proof that the physical model faces right while moving
        // right.
        if [30, 77, 86, 100]
            .get(target as usize)
            .is_some_and(|minimum| frame >= *minimum)
        {
            screenshot();
            self.captured.set(target + 1);
            #[cfg(not(feature = "headless-integration"))]
            if target == 3 {
                self.quit_frame.set(160);
            }
        }
    }
}
