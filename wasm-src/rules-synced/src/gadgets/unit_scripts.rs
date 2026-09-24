use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use spring_addons::rules::Gadget;

use crate::state::Global;

/// Attaches each unit's script in the unit-script module.
#[derive(Default)]
pub struct UnitScriptsGadget {
    /// Whether the map's own units, which predate this module, got theirs.
    swept: Cell<bool>,
    /// Units created while the unit-script module was unavailable: before the
    /// engine initialized it, or while it was busy further up the stack (a
    /// script whose engine call raised UnitCreated).  Attached next frame.
    pending: RefCell<Vec<(i32, i32)>>,
}

impl UnitScriptsGadget {
    pub fn new() -> Self {
        Self::default()
    }

    fn attach(&self, unit: i32, def: i32) {
        if crate::with_cus_module(|module| module.attach(unit, def)).is_none() {
            self.pending.borrow_mut().push((unit, def));
        }
    }
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        let mut units = core::mem::take(&mut *self.pending.borrow_mut());
        if !self.swept.replace(true) {
            for id in spring::get_all_units().unwrap_or_default() {
                if let Ok(def) = spring::get_unit_def_id(id) {
                    units.push((id, def.0));
                }
            }
        }
        for (id, def) in units {
            if spring::valid_unit_id(id).unwrap_or(false) {
                self.attach(id, def);
            }
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
        self.attach(id, def);
    }
}
