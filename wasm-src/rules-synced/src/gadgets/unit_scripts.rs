use core::cell::Cell;

use spring::UnitId;
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Attaches each unit's script in the unit-script module.
#[derive(Default)]
pub struct UnitScriptsGadget {
    started: Cell<bool>,
}

impl UnitScriptsGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    /// Units created before the game starts (the map's own units) get their
    /// scripts on the first frame. Before it `CusHost` would start their
    /// clocks at frame -1 read as `u32::MAX`, and a sleep in their Create
    /// would never wake (spring-bar WASM_GAME_API_TODO_2026-09-24.md §I.1).
    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        if !self.started.replace(true) {
            crate::with_cus_module_or_defer(|host| host.attach_existing());
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        id: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        if self.started.get() {
            crate::with_cus_module_or_defer(move |host| host.unit_created(UnitId(id), def));
        }
    }
}
