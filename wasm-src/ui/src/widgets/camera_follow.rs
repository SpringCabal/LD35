//! The tracking camera, ported from `camera_control.lua`.

use core::cell::{Cell, RefCell};

use spring_addons::event::EventResult;
use spring_addons::ui::Widget;

use crate::state::Global;

// Match camera_control.lua. The target is smoothed in the widget; the engine
// transition itself stays at zero so the two interpolation layers do not fight
// each other.
const CAMERA_FOLLOW_TAU: f32 = 0.12;

#[derive(Clone, Copy)]
struct CameraTracking {
    wisp_id: i32,
    focus: spring::Float3,
}

/// Places the authored camera, then keeps it on the player wisp.
#[derive(Default)]
pub struct CameraFollowWidget {
    placed: Cell<bool>,
    tracking: RefCell<Option<CameraTracking>>,
}

impl CameraFollowWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn place(&self) {
        let _ = spring::set_camera_state(
            spring::CameraState {
                name: "spring",
                position: spring::Float3::new(1078.2821, 436.30078, 2710.0608),
                direction: spring::Float3::new(0.0, -0.8283768, -0.5601712),
                up: spring::Float3::new(0.0, 0.5601712, -0.8283768),
                right: spring::Float3::new(1.0, 0.0, 0.0),
                fov: 45.0,
                rx: 2.547,
                ry: 0.01,
                rz: 0.0,
                dist: 518.5416,
                height: 0.0,
                angle: 0.0,
                mode: 2,
            },
            0.0,
            1.0,
            1.0,
        );
    }

    fn lock_camera(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let wisp = ctx.global().player.access().wisp;
        if wisp < 0 {
            return;
        }
        // camera_control.lua follows the unit's view position: the raw physics
        // position moves in simulation steps and would make the camera jitter.
        let Ok(position) = spring::unit_rendering::get_unit_view_position(wisp, false) else {
            return;
        };
        let target = position;

        let focus = {
            let mut tracking = self.tracking.borrow_mut();
            // Clamp the first sample (and every newly discovered UnitCreated
            // equivalent) directly to the live unit. Interpolating from the
            // zero/default point would cause a visible camera sweep at start.
            let tracking = match tracking.as_mut() {
                Some(tracking) if tracking.wisp_id == wisp => tracking,
                _ => tracking.insert(CameraTracking {
                    wisp_id: wisp,
                    focus: target,
                }),
            };
            if dt.is_finite() && dt > 0.0 {
                let alpha = 1.0 - libm::expf(-(dt / CAMERA_FOLLOW_TAU));
                tracking.focus.x += (target.x - tracking.focus.x) * alpha;
                tracking.focus.y += (target.y - tracking.focus.y) * alpha;
                tracking.focus.z += (target.z - tracking.focus.z) * alpha;
            }
            tracking.focus
        };

        // Lua uses SetCameraTarget(..., 0): the smoothing above is the only
        // interpolation. Supplying no transition time would add the engine's
        // default 0.5-second transition on top and make the camera laggy.
        let _ = spring::camera::set_camera_target(
            focus,
            spring::camera::SetCameraTargetOptions {
                transition_time: Some(0.0),
                dir_x: None,
                dir_y: None,
                dir_z: None,
            },
        );
    }
}

impl Widget<Global> for CameraFollowWidget {
    fn name(&self) -> &'static str {
        "CameraFollow"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        if !self.placed.replace(true) {
            self.place();
        }
        self.lock_camera(ctx, dt);
    }

    fn mouse_wheel(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _up: bool,
        _value: f32,
    ) -> EventResult {
        // The authored camera is fixed. Claim the wheel at the input receiver
        // boundary so the engine never applies its default zoom operation.
        EventResult::Handled
    }
}
