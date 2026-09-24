use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::defs;
use crate::state::Global;

/// Keeps track of the player wisp, the body-part pickups and the NPC wisps,
/// and gives each cage its prisoner (unit_cage.lua).
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
        let global = ctx.global();
        let defs = defs();
        if def == defs.wisp {
            let mut wisp = global.wisp.access_mut();
            wisp.id = id;
            wisp.pieces_dirty = true;
        } else if defs.body_part(def).is_some() {
            global.wisp.access_mut().body_parts.push(id);
        } else if def == defs.npcwisp {
            global.world.access_mut().npc_wisps.push(id);
        } else if def == defs.cage {
            // Creating the prisoner raises UnitCreated again.
            ctx.delay(move |_| crate::cages::spawn_cage_wisp(id));
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
