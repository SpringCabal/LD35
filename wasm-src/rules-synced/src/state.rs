//! Shared rules state.
//!
//! Everything the gadgets read and write lives here; the handler owns it.

use alloc::string::String;
use alloc::vec::Vec;

use spring_addons::Resource;

pub const PUBLIC: i32 = 32;
pub const FORM_COOLDOWN: i32 = 60;
pub const WISP_DESTROY_DELAY: i32 = 45;
// The Lua death gadget uses 45 frames for ordinary rescues. Ending wisps are
// spawned hundreds of elmos away and must be allowed to reach the player before
// their saved-wisp fade begins; keep that longer ceremony lifetime separate so
// normal cage rescues still match Lua.
pub const ENDING_WISP_DESTROY_DELAY: i32 = 180;

/// The player wisp: its identity, how it is currently moving, and which body
/// parts it has collected.
#[derive(Default)]
pub struct Wisp {
    pub id: i32,
    pub movement: Option<(f32, f32)>,
    pub movement_frame: i32,
    pub stop_issued: bool,
    pub script_moving: bool,
    pub form_frame: i32,
    pub spirit: bool,
    pub has_eyes: bool,
    pub body_parts: Vec<i32>,
    pub collected_parts: Vec<String>,
    pub pieces_dirty: bool,
}

/// The level: cages, the NPC wisps inside them, and the trigger network.
#[derive(Default)]
pub struct World {
    pub cages: Vec<i32>,
    pub npc_wisps: Vec<i32>,
    pub level_requested: bool,
    pub level_ids: Vec<(i32, i32)>,
    pub active_triggers: Vec<i32>,
    pub active_triggerables: Vec<i32>,
}

/// End-of-game sequencing and the fate of each NPC wisp.
#[derive(Default)]
pub struct Endgame {
    pub game_over: bool,
    pub ending_type: i32,
    pub dying_wisps: Vec<(i32, bool, i32)>,
    pub saved_wisps: i32,
    pub killed_wisps: i32,
    pub release_wisps: bool,
    pub end_spawned: i32,
    pub end_spawn_frame: i32,
    /// Ceremonial spirits and the position each one should converge on. The
    /// normal wisp death lifecycle removes them after their short entrance.
    pub ending_wisps: Vec<(i32, spring::Float3)>,
    /// The visual ending is complete once the authored batch has been issued.
    pub sequence_complete: bool,
}

/// Frame bookkeeping.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    pub initialized: bool,
    pub frame: i32,
}

/// Work deferred out of a nested callin and drained by the reaper gadget.
///
/// Core applies a unit's ON/OFF state but does not dispatch the legacy
/// COB/Lua Activate callback, so those callbacks are queued here; `cage.lua`
/// can then send its `npc_wisp` result back through RecvLuaMsg without a
/// borrow still being held.
#[derive(Default)]
pub struct Pending {
    pub cage_activations: Vec<i32>,
    pub piece_hides: Vec<i32>,
    pub destroys: Vec<(i32, bool, i32)>,
    pub spawn_level: bool,
    pub spawn_end_wisp: bool,
    pub stop_wisp: Option<i32>,
    pub animation: Option<(i32, bool)>,
    pub wisp_pieces: Option<(i32, bool, Vec<String>)>,
}

pub fn param(name: &str, value: f32) {
    let _ = spring::set_game_rules_param(
        name,
        &spring::owned::rules_params::RulesParamValue {
            type_: spring::owned::rules_params::RulesParamType::RulesparamTypeFloat,
            bool_value: false,
            float_value: value,
            string_value: String::new(),
        },
        PUBLIC,
    );
}

/// The shared communication namespace.
///
/// Each subsystem is an independently borrowable [`Resource`], so a nested
/// callin only conflicts when it touches the same subsystem.
#[derive(Default)]
pub struct Global {
    pub wisp: Resource<Wisp>,
    pub world: Resource<World>,
    pub endgame: Resource<Endgame>,
    pub clock: Resource<Clock>,
    pub pending: Resource<Pending>,
}
