use alloc::vec::Vec;

use crate::defs::{defs, unit_def};

const SCAN_INTERVAL: i32 = 30;

#[derive(Default)]
pub struct Npcs {
    pub wisps: Vec<i32>,
    hidden: Vec<i32>,
    scan_ticks: i32,
}

impl Npcs {
    pub fn refresh(&mut self, spring: &spring::Spring<'_, spring::Ui>, ending: bool) {
        if !ending && self.scan_ticks > 0 {
            self.scan_ticks -= 1;
            return;
        }
        self.scan_ticks = if ending { 0 } else { SCAN_INTERVAL };
        let Ok(all_units) = spring.units_query().get_all_units() else {
            return;
        };
        let defs = defs(spring);
        let mut wisps = Vec::new();
        let mut still_hidden = Vec::with_capacity(self.hidden.len());
        for unit_id in all_units {
            let def = unit_def(spring, unit_id);
            if def == defs.npcwisp {
                wisps.push(unit_id);
            } else if def != defs.light {
                continue;
            }
            if self.hidden.binary_search(&unit_id).is_err() {
                let _ = spring.unsynced_ctrl().set_unit_no_draw(unit_id, true);
            }
            still_hidden.push(unit_id);
        }
        still_hidden.sort_unstable();
        self.hidden = still_hidden;
        self.wisps = wisps;
    }
}
