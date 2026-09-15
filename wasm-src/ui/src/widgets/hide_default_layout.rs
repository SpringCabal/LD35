use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct HideDefaultLayoutWidget {
    applied: core::cell::Cell<bool>,
}

impl HideDefaultLayoutWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for HideDefaultLayoutWidget {
    fn name(&self) -> &'static str {
        "HideDefaultLayout"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        if self.applied.get() {
            return;
        }
        self.applied.set(true);
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
        let commands_hidden = [
            "Console 0",
            "ResBar 0",
            "ToolTip 0",
            "Clock 0",
            "Info 0",
            "showhealthbars 0",
            "showrezbars 0",
            "disticon 99999",
        ]
        .into_iter()
        .all(|command| spring::send_commands(command, "").is_ok());
        let minimap_hidden =
            spring::slave_mini_map(true).is_ok() && spring::config_mini_map(-1, -1, -1, -1).is_ok();
        let _ = (commands_hidden, minimap_hidden);
    }
}
