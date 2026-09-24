use core::cell::Cell;

use alloc::vec::Vec;
use partsneeded_shared::Command;
use spring_addons::rules::Gadget;

use crate::state::{Endgame, Global, Wisp, World};

/// Frame the level is spawned on, after the map's own units exist.
const SPAWN_FRAME: i32 = 3;

/// game_custom_spawn.lua rebuilds the level 3 frames after it cleared it.
const RESPAWN_DELAY: i32 = 3;

/// Spawns the level from game_custom_spawn.lua's tables, and spawns it again
/// when the player restarts from the end screen.
///
/// ui_endgame.lua's Restart runs `luarules reload`: every gadget starts
/// over, and game_custom_spawn.lua destroys all but the map's scenery and
/// builds the level again. That reloads only Lua gadgets, not this module,
/// so the rules do the same themselves.
pub struct LevelGadget {
    /// The frame the level is (re)spawned on; `None` once it is.
    spawn_frame: Cell<Option<i32>>,
    restart_requested: Cell<bool>,
}

impl Default for LevelGadget {
    fn default() -> Self {
        Self {
            spawn_frame: Cell::new(Some(SPAWN_FRAME)),
            restart_requested: Cell::new(false),
        }
    }
}

impl LevelGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for LevelGadget {
    fn name(&self) -> &'static str {
        "Level"
    }

    /// The UI offers the restart on the end screen only; the rules take it
    /// only once the game is over.
    fn handle_lua_msg(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let restart = core::str::from_utf8(data)
            .ok()
            .and_then(Command::decode)
            .is_some_and(|command| command == Command::Restart);
        if restart && ctx.global().endgame.access().game_over {
            self.restart_requested.set(true);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if self.restart_requested.take() && self.spawn_frame.get().is_none() {
            spring::log_info!(crate::log::SECTION, "the level restarts at frame {frame}");
            self.spawn_frame.set(Some(frame + RESPAWN_DELAY));
            // Destroying raises UnitDestroyed, which needs the state.
            ctx.delay(clear_level);
        }
        if self.spawn_frame.get().is_some_and(|at| frame >= at) {
            self.spawn_frame.set(None);
            // Creating units raises UnitCreated, which needs the world.
            ctx.delay(move |global| {
                let ids = crate::level::spawn();
                let mut world = global.world.access_mut();
                world.level_ids = ids.into_iter().collect();
                world.spawn_frame = frame;
                crate::triggers::initialize_gate_terrain(&world);
            });
        }
    }
}

/// game_custom_spawn.lua's clear: destroy every unit but the map's scenery
/// (the gates too: they are built again closed), then forget the round.
fn clear_level(global: &Global) {
    let units: Vec<i32> = spring::get_all_units()
        .unwrap_or_default()
        .into_iter()
        .filter(|id| {
            let def = crate::defs::unit_def(*id);
            def == crate::defs::defs().gate || !crate::defs::params(def).dungeon_element
        })
        .collect();
    for id in units {
        crate::log::check(
            "restart: destroy",
            id,
            spring::destroy_unit(
                id,
                spring::rules_synced::unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed: false,
                    attacker_id: -1,
                    recycle_id: false,
                },
            ),
        );
    }
    *global.wisp.access_mut() = Wisp::default();
    *global.world.access_mut() = World::default();
    *global.endgame.access_mut() = Endgame::default();
}
