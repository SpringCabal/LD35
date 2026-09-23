//! Cages and the NPC wisps they hold, ported from `cage.lua`.

use alloc::vec::Vec;

use crate::endgame::mark_wisp_dying;
use crate::state::Global;
use crate::triggers::nearby;

pub fn spawn_cage_wisp(cage: i32) {
    let Ok(position) = spring::get_unit_position(cage, false, false) else {
        return;
    };
    let _ = crate::level::spawn_unit("npcwisp", crate::defs::defs().npcwisp, position);
}

pub fn activate_cage(global: &Global, cage: i32) {
    let kill = spring::get_unit_rules_param_f32(cage, "do_kill") == Ok(1.0);
    let nearby_wisps: Vec<i32> = global
        .world
        .access()
        .npc_wisps
        .iter()
        .copied()
        .filter(|wisp| nearby(cage, *wisp, 50.0))
        .collect();
    for wisp in nearby_wisps {
        mark_wisp_dying(global, wisp, !kill);
    }
}
