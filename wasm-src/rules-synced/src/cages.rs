use alloc::vec::Vec;

use crate::gadgets::endgame::behavior::mark_wisp_dying;
use crate::gadgets::triggers::behavior::nearby;
use crate::state::Global;

pub const BARS_TRAVEL: f32 = 200.0;
pub const BARS_SPEED: f32 = 150.0;

pub const OPEN_FRAMES: i32 = 40;

const PRISONER_RADIUS: f32 = 50.0;

pub fn spawn_cage_wisp(cage: i32) {
    let Ok(position) = spring::get_unit_position(cage, false, false) else {
        return;
    };
    crate::level::spawn_unit("npcwisp", crate::defs::defs().npcwisp, position, |_| {});
}

pub fn kills(cage: i32) -> bool {
    spring::get_unit_rules_param_f32(cage, "do_kill") == Ok(1.0)
}

pub fn resolve_cage(global: &Global, cage: i32, kill: bool) {
    let nearby_wisps: Vec<i32> = global
        .world
        .access()
        .npc_wisps
        .iter()
        .copied()
        .filter(|wisp| nearby(cage, *wisp, PRISONER_RADIUS))
        .collect();
    for wisp in nearby_wisps {
        mark_wisp_dying(global, wisp, !kill);
    }
}
