use alloc::vec::Vec;
use core::cell::RefCell;

use spring_addons::rules::Gadget;

use crate::cages;
use crate::state::Global;
use crate::triggers;

/// unit_plate.lua's UPDATE_RATE: the trigger network is evaluated every this
/// many frames.
const UPDATE_RATE: i32 = 5;

/// Plates, levers and the doors and cages they open, ported from
/// `unit_plate.lua` and `cage.lua`.
#[derive(Default)]
pub struct TriggersGadget {
    /// Opened cages whose prisoner is freed or killed on a later frame:
    /// (cage, frame, kill).
    opening_cages: RefCell<Vec<(i32, i32, bool)>>,
}

impl TriggersGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for TriggersGadget {
    fn name(&self) -> &'static str {
        "Triggers"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        if frame % UPDATE_RATE == 0 {
            let opened = triggers::update_triggers(global);
            let mut opening = self.opening_cages.borrow_mut();
            for cage in opened {
                let kill = cages::kills(cage);
                let due = if kill {
                    frame
                } else {
                    frame + cages::OPEN_FRAMES
                };
                opening.push((cage, due, kill));
            }
        }
        let due: Vec<(i32, bool)> = {
            let mut opening = self.opening_cages.borrow_mut();
            let due = opening
                .iter()
                .filter(|(_, at, _)| *at <= frame)
                .map(|(cage, _, kill)| (*cage, *kill))
                .collect();
            opening.retain(|(_, at, _)| *at > frame);
            due
        };
        for (cage, kill) in due {
            cages::resolve_cage(global, cage, kill);
        }
    }
}
