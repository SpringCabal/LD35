use spring::Float3;
use spring_addons::event::EventResult;
use spring_addons::ui::Widget;
use springcabal_control::camera::{Aim, CameraFollow, Follow, Smoothing, View};

use crate::state::Global;

const START_VIEW: View = View::spring(518.5416, 2.547)
    .ry(0.01)
    .pitch(-0.828_376_8, -0.560_171_2);
const START_POSITION: Float3 = Float3::new(1078.2821, 436.30078, 2710.0608);

const FOLLOW: Follow = Follow::target(0.0)
    .start(START_VIEW, START_POSITION)
    .aim(Aim::ViewPosition)
    .smoothing(Smoothing::Exponential { tau: 0.12 });

pub struct CameraFollowWidget {
    camera: CameraFollow,
}

impl Default for CameraFollowWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraFollowWidget {
    pub fn new() -> Self {
        Self {
            camera: CameraFollow::new(FOLLOW),
        }
    }
}

impl Widget<Global> for CameraFollowWidget {
    fn name(&self) -> &'static str {
        "CameraFollow"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let wisp = ctx.global().player.access().wisp;
        self.camera.update((wisp >= 0).then_some(wisp), dt);
    }

    fn mouse_wheel(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _up: bool,
        _value: f32,
    ) -> EventResult {
        EventResult::Handled
    }
}
