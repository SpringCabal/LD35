use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::defs;
use crate::state::Global;

#[derive(Default)]
pub struct RosterGadget;

impl RosterGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for RosterGadget {
    fn name(&self) -> &'static str {
        "Roster"
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
            crate::with_addon_context(|ctx| {
                ctx.commands()
                    .run(move |spring| crate::cages::spawn_cage_wisp(spring, id))
            });
        }
    }

    fn unit_destroyed(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        let global = ctx.global();
        let id = event.unit_id;
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
