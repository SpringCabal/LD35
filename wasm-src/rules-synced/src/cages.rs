//! Cages and the NPC wisps they hold, ported from `cage.lua`.

use alloc::vec::Vec;

use crate::endgame::mark_wisp_dying;
use crate::state::Global;
use crate::triggers::nearby;

/// How far cage.lua slides the front bars open, and how fast (elmos/s).
pub const BARS_TRAVEL: f32 = 200.0;
pub const BARS_SPEED: f32 = 150.0;

/// cage.lua frees its prisoner once the bars have finished opening: 200
/// elmos at 150 elmos/s is 40 frames.  A killing cage doesn't open its bars
/// and resolves at once.
pub const OPEN_FRAMES: i32 = 40;

/// cage.lua looks for its prisoner within this radius.
const PRISONER_RADIUS: f32 = 50.0;

pub fn spawn_cage_wisp(cage: i32) {
    let Ok(position) = spring::get_unit_position(cage, false, false) else {
        return;
    };
    crate::level::spawn_unit("npcwisp", crate::defs::defs().npcwisp, position, |_| {});
}

/// Whether a cage opening now kills its prisoner: a kill link has set
/// `do_kill` on it.  cage.lua reads it once, when it activates.
pub fn kills(cage: i32) -> bool {
    spring::get_unit_rules_param_f32(cage, "do_kill") == Ok(1.0)
}

/// Save or kill the NPC wisps in an opened cage.
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
