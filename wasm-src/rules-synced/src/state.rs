use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use partsneeded_shared::{BodyParts, Ending};
use spring_addons::Resource;
use springcabal_control::goal::Steering;

pub const PUBLIC: i32 = 32;
pub use partsneeded_shared::FORM_COOLDOWN;
pub const WISP_DESTROY_DELAY: i32 = 45;
pub const ENDING_WISP_DESTROY_DELAY: i32 = 180;

pub struct Wisp {
    pub id: i32,
    pub steering: Steering,
    pub form_frame: i32,
    pub spirit: bool,
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

#[derive(Default)]
pub struct World {
    pub npc_wisps: Vec<i32>,
    pub level_ids: BTreeMap<i32, i32>,
    pub active_triggers: Vec<i32>,
    pub active_triggerables: Vec<i32>,
    pub spawn_frame: i32,
}

#[derive(Default)]
pub struct Endgame {
    pub game_over: bool,
    pub ending: Option<Ending>,
    pub dying_wisps: Vec<(i32, bool, i32)>,
    pub saved_wisps: i32,
    pub killed_wisps: i32,
    pub release_wisps: bool,
    pub end_spawned: i32,
    pub end_spawn_frame: i32,
    pub ending_wisps: Vec<(i32, spring::Float3)>,
    pub sequence_complete: bool,
}

#[derive(Default)]
pub struct Global {
    pub wisp: Resource<Wisp>,
    pub world: Resource<World>,
    pub endgame: Resource<Endgame>,
}
