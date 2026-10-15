use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct UnitScriptProbeWidget {
    begin_sent: core::cell::Cell<bool>,
}

impl UnitScriptProbeWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for UnitScriptProbeWidget {
    fn name(&self) -> &'static str {
        "UnitScriptProbe"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        let _ = spring::echo("[UnitScriptProbeDriver] LOADED script=WasmUI", "");
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        if !self.begin_sent.replace(true) {
            let _ = spring::echo("[UnitScriptProbeDriver] BEGIN", "");
            springcabal::message::send_to_rules("probe_begin");
        }
    }
}
