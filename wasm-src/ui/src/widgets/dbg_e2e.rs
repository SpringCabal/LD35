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

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let Ok(counter) = spring::game::get_game_frame(0) else {
            return;
        };
        let frame = (counter.low16 | (counter.high16 << 16)) as i32;
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
            let _ = spring::send_commands("screenshot", "");
            let _ = spring::send_commands("console", "0");
            return;
        }
        let ending_start = self.ending_start.get();
        if ending_start >= 0
            && frame - ending_start >= 180
            && !self.ending_final_captured.replace(true)
        {
            let _ = spring::send_commands("screenshot", "");
            let _ = spring::send_commands("console", "0");
            self.quit_frame.set(frame + 30);
            return;
        }
        let target = self.captured.get();
        if [30, 65, 77, 86, 100]
            .get(target as usize)
            .is_some_and(|minimum| frame >= *minimum)
        {
            let _ = spring::send_commands("screenshot", "");
            let _ = spring::send_commands("console", "0");
            self.captured.set(target + 1);
            #[cfg(not(feature = "headless-integration"))]
            if target == 3 {
                self.quit_frame.set(160);
            }
        }
    }
}
