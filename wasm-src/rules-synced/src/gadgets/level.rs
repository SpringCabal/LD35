use core::cell::Cell;

use alloc::vec::Vec;
use partsneeded_shared::Command;

use spring_addons::rules::Gadget;

use crate::state::{Endgame, Global, Wisp, World};

const SPAWN_FRAME: i32 = 3;

const RESPAWN_DELAY: i32 = 3;

pub struct LevelGadget {
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

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = _ctx.spring();
        if self.restart_requested.take() && self.spawn_frame.get().is_none() {
            spring::log_info!(
                spring,
                crate::log::SECTION,
                "the level restarts at frame {frame}"
            );
            self.spawn_frame.set(Some(frame + RESPAWN_DELAY));
            clear_level(spring);
        }
        if self.spawn_frame.get().is_some_and(|at| frame >= at) {
            self.spawn_frame.set(None);
            crate::level::spawn(spring, move |spring, ids| {
                crate::with_global(|global| {
                    let mut world = global.world.access_mut();
                    world.level_ids = ids.into_iter().collect();
                    world.spawn_frame = frame;
                    crate::gadgets::triggers::behavior::initialize_gate_terrain(spring, &world);
                });
            });
        }
    }
}

fn clear_level(spring: &spring::Spring<'_, spring::RulesSynced>) {
    let units: Vec<i32> = spring
        .units()
        .get_all_units()
        .unwrap_or_default()
        .into_iter()
        .filter(|id| {
            let def = crate::defs::unit_def(spring, *id);
            def == crate::defs::defs(spring).gate
                || !crate::defs::params(spring, def).dungeon_element
        })
        .collect();
    for id in units {
        crate::with_addon_context(|ctx| {
            ctx.commands().destroy_unit(
                id,
                spring::unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed: false,
                    attacker_id: -1,
                    recycle_id: false,
                },
            )
        })
        .then(move |spring, result| {
            crate::log::check(spring, "restart: destroy", id, result);
        });
    }
    crate::with_addon_context(|ctx| {
        ctx.commands().run(|_spring| {
            crate::with_global(|global| {
                *global.wisp.access_mut() = Wisp::default();
                *global.world.access_mut() = World::default();
                *global.endgame.access_mut() = Endgame::default();
            });
        })
    });
}
