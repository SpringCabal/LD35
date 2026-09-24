//! The NPC wisps the billboards are drawn for, and the units drawn as nothing.

use alloc::vec::Vec;

use crate::defs::{defs, unit_def};

/// Update ticks between scans outside the ending.
const SCAN_INTERVAL: i32 = 30;

#[derive(Default)]
pub struct Npcs {
    pub wisps: Vec<i32>,
    /// NPC wisps and lights already set to no-draw, sorted.
    hidden: Vec<i32>,
    scan_ticks: i32,
}

impl Npcs {
    pub fn refresh(&mut self, ending: bool) {
        // During the ending new ceremonial wisps must enter the billboard list
        // immediately. The normal scan interval is enough in the dungeon, but
        // made each arrival briefly show its physical model.
        if !ending && self.scan_ticks > 0 {
            self.scan_ticks -= 1;
            return;
        }
        self.scan_ticks = if ending { 0 } else { SCAN_INTERVAL };
        let Ok(all_units) = spring::get_all_units() else {
            return;
        };
        let defs = defs();
        let mut wisps = Vec::new();
        let mut still_hidden = Vec::with_capacity(self.hidden.len());
        for unit_id in all_units {
            let def = unit_def(unit_id);
            if def == defs.npcwisp {
                wisps.push(unit_id);
            } else if def != defs.light {
                continue;
            }
            // NPC wisps are drawn as billboards, and unit_wisp_draw.lua hides
            // every light unit: they exist to place a deferred light.  Hide
            // each one once, when it first shows up.
            if self.hidden.binary_search(&unit_id).is_err() {
                let _ = spring::unsynced_ctrl::set_unit_no_draw(unit_id, true);
            }
            still_hidden.push(unit_id);
        }
        still_hidden.sort_unstable();
        self.hidden = still_hidden;
        self.wisps = wisps;
    }
}
