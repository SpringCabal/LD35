use alloc::vec::Vec;
use core::cell::RefCell;

use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::defs;
use crate::state::Global;

#[derive(Default)]
pub struct RosterGadget {
    pending_cages: RefCell<Vec<i32>>,
}

impl RosterGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for RosterGadget {
    fn name(&self) -> &'static str {
        "Roster"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        // Match Lua's next-GameFrame cage births, in a stable shared order.
        let mut pending = core::mem::take(&mut *self.pending_cages.borrow_mut());
        pending.sort_unstable();
        for id in pending {
            ctx.commands()
                .run(move |spring| crate::cages::spawn_cage_wisp(spring, id));
        }
    }

    fn unit_created(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        id: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = ctx.spring();
        let global = ctx.global();
        let defs = defs(spring);
        if def == defs.wisp {
            let mut wisp = global.wisp.access_mut();
            wisp.id = id;
            wisp.pieces_dirty = true;
        } else if defs.body_part(def).is_some() {
            global.wisp.access_mut().body_parts.push(id);
        } else if def == defs.npcwisp {
            global.world.access_mut().npc_wisps.push(id);
        } else if def == defs.cage {
            self.pending_cages.borrow_mut().push(id);
        }
    }

    fn unit_destroyed(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        let global = ctx.global();
        let id = event.unit_id;
        self.pending_cages.borrow_mut().retain(|cage| *cage != id);
        {
            let mut wisp = global.wisp.access_mut();
            if wisp.id == id {
                wisp.id = -1;
            }
            wisp.body_parts.retain(|part| *part != id);
        }
        global
            .world
            .access_mut()
            .npc_wisps
            .retain(|wisp| *wisp != id);
    }
}
