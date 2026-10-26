use alloc::vec::Vec;

use crate::defs::{defs, unit_def};

#[derive(Default)]
pub struct Npcs {
    pub wisps: Vec<i32>,
    initialized: bool,
}

impl Npcs {
    pub fn refresh(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        if self.initialized {
            return;
        }
        let Ok(all_units) = spring.units_query().get_all_units() else {
            return;
        };
        self.initialized = true;
        for id in all_units {
            self.track(spring, id, unit_def(spring, id));
        }
    }

    pub fn track(&mut self, spring: &spring::Spring<'_, spring::Ui>, id: i32, def: i32) {
        let defs = defs(spring);
        if def == defs.npcwisp {
            if let Err(index) = self.wisps.binary_search(&id) {
                self.wisps.insert(index, id);
            }
        } else if def != defs.light {
            return;
        }
        let _ = spring.unsynced_ctrl().set_unit_no_draw(id, true);
    }

    pub fn remove(&mut self, id: i32) {
        if let Ok(index) = self.wisps.binary_search(&id) {
            self.wisps.remove(index);
        }
    }
}
