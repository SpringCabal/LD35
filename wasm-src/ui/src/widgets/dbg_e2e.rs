use core::cell::Cell;

use partsneeded_shared::Ending;
use spring_addons::ui::Widget;
use springcabal::shots::{Clock, Screenshots, screenshot};

use crate::state::Global;

const SHOT_FRAMES: [u32; 2] = [30, 84];

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

const SETTLED_STEP: f32 = 0.5;
const SETTLED_UPDATES: u32 = 3;

#[cfg(not(feature = "headless-integration"))]
const QUIT_AFTER_LAST_SHOT: i32 = 60;

struct SettledShot {
    label: &'static str,
    from: i32,
    until: i32,
}

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

    fn track_camera(&self, spring: &spring::Spring<'_, spring::Ui>) -> u32 {
        let Ok(now) = spring.camera().get_camera_position() else {
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

    fn settled_shot(&self, spring: &spring::Spring<'_, spring::Ui>, frame: i32) {
        let still_updates = self.track_camera(spring);
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
                spring,
                "partsneeded",
                "E2E camera not settled for screenshot {}",
                shot.label
            );
        }
        screenshot(spring, &alloc::format!("{} at {frame}", shot.label));
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

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        let spring = ctx.spring();
        let frame = spring.game().current_frame().unwrap_or(0);
        let quit_frame = self.quit_frame.get();
        if quit_frame >= 0 && frame >= quit_frame {
            let _ = spring.system_control().quit();
            return;
        }
        let ending = ctx.global().player.access();
        if ending.ending == Some(Ending::SoulsFreed)
            && ending.game_over_sequence
            && self.ending_start.get() < 0
        {
            self.ending_start.set(frame);
            screenshot(spring, "ending start");
            return;
        }
        drop(ending);
        let ending_start = self.ending_start.get();
        if ending_start >= 0
            && frame - ending_start >= ENDING_FINAL_SHOT_DELAY
            && !self.ending_final_captured.replace(true)
        {
            screenshot(spring, "ending final");
            self.quit_frame.set(frame + 30);
            return;
        }
        self.shots.update(ctx, dt);
        self.settled_shot(spring, frame);
    }
}
