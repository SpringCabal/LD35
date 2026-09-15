use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::{cages, endgame, level, triggers, wisp};

#[derive(Default)]
pub struct ReaperGadget;

impl ReaperGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for ReaperGadget {
    fn name(&self) -> &'static str {
        "Reaper"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        // Take every queued action first: the callouts below re-enter callins
        // that need this same state.
        let (
            piece_hides,
            wisp_pieces,
            stop_wisp,
            cage_activations,
            spawn_level,
            cages,
            spawn_end_wisp,
            destroys,
        ) = {
            let mut pending = global.pending.access_mut();
            (
                core::mem::take(&mut pending.piece_hides),
                pending.wisp_pieces.take(),
                pending.stop_wisp.take(),
                core::mem::take(&mut pending.cage_activations),
                core::mem::take(&mut pending.spawn_level),
                core::mem::take(&mut global.world.access_mut().cages),
                core::mem::take(&mut pending.spawn_end_wisp),
                core::mem::take(&mut pending.destroys),
            )
        };

        for id in piece_hides {
            wisp::hide_all_wisp_pieces(id);
        }
        if let Some((id, spirit, parts)) = wisp_pieces {
            wisp::sync_wisp_pieces(id, spirit, &parts);
        }
        if let Some(id) = stop_wisp {
            let _ = spring::give_order_to_unit(id, 0, &[], 0, 0);
        }
        for cage in cage_activations {
            cages::activate_cage(global, cage);
        }
        if spawn_level {
            let ids = level::spawn();
            {
                let mut world = global.world.access_mut();
                world.level_ids = ids;
                triggers::initialize_gate_terrain(&world);
            }
        }
        for cage in cages {
            cages::spawn_cage_wisp(cage);
        }
        if spawn_end_wisp {
            endgame::spawn_ending_wisp(global, frame);
        }
        for (id, reclaimed, attacker) in destroys {
            let _ = spring::destroy_unit(
                id,
                spring::rules_synced::unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed,
                    attacker_id: attacker,
                    recycle_id: false,
                },
            );
        }
    }
}
