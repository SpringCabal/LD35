use spring::UnitId;
use spring_addons::rules::Gadget;

use crate::state::Global;

#[derive(Default)]
pub struct UnitScriptsGadget;

impl UnitScriptsGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        crate::unit_scripts::with_host(|host| host.attach_existing());
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        crate::unit_scripts::with_host(move |host| host.game_frame(frame));
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        id: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        crate::unit_scripts::with_host(move |host| host.unit_created(UnitId(id), def));
    }
}
