//! Rules state that more than one gadget reads or writes.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use partsneeded_shared::BodyParts;
use spring_addons::Resource;
use springcabal_control::goal::Steering;

pub const PUBLIC: i32 = 32;
pub use partsneeded_shared::FORM_COOLDOWN;
pub const WISP_DESTROY_DELAY: i32 = 45;
// The Lua death gadget uses 45 frames for ordinary rescues. Ending wisps are
// spawned hundreds of elmos away and must be allowed to reach the player before
// their saved-wisp fade begins; keep that longer ceremony lifetime separate so
// normal cage rescues still match Lua.
pub const ENDING_WISP_DESTROY_DELAY: i32 = 180;

/// The player wisp: its identity, how it is currently moving, and which body
/// parts it has collected.
pub struct Wisp {
    pub id: i32,
    /// The held movement keys.
    pub steering: Steering,
    pub form_frame: i32,
    pub spirit: bool,
    /// The body-part pickups still lying in the level.
    pub body_parts: Vec<i32>,
    pub collected_parts: BodyParts,
    pub pieces_dirty: bool,
}

impl Default for Wisp {
    fn default() -> Self {
        Self {
            id: -1,
            steering: Steering::new(),
            form_frame: 0,
            spirit: true,
            body_parts: Vec::new(),
            collected_parts: BodyParts::NONE,
            pieces_dirty: false,
        }
    }
}

/// The level: the NPC wisps and the trigger network.
#[derive(Default)]
pub struct World {
    pub npc_wisps: Vec<i32>,
    /// Engine unit id by map_customs.lua content key.
    pub level_ids: BTreeMap<i32, i32>,
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

/// The current simulation frame.
pub fn frame() -> i32 {
    spring::game::frame().map_or(0, |frame| frame as i32)
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
}
