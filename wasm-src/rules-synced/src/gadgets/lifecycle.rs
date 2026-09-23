use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{collision, defs};
use crate::state::{Global, PUBLIC, param};
use crate::unit_scripts;

/// Keep map-authored walls at the level's explicit floor height.  A map unit
/// can predate the WASM rules module, so it does not necessarily pass through
/// level::create(), where the same MoveCtrl pin is applied.
fn pin_authored_wall_height(id: i32, def: i32) {
    if !collision(def).pinned_wall {
        return;
    }
    let Ok(position) = spring::get_unit_position(id, false, false) else {
        return;
    };
    let _ = spring::move_ctrl::move_ctrl(id, true);
    let _ = spring::move_ctrl::set_move_ctrl_gravity(id, 0.0);
    let _ = spring::set_unit_position(
        id,
        spring::Float3::new(position.x, crate::level::FLOOR_Y, position.z),
    );
}

/// Everything unit_collisions.lua and unit_plate.lua do to a unit's collision
/// state the moment it is created.
///
/// The dungeon's walls, levers and plates are scenery: the real barriers are
/// the map's terrain, and the authored gadgets take all three out of the
/// collision world.  Leaving them solid means the player is shoved around by
/// decoration, and a prisoner spawned at its cage would be displaced to the
/// nearest free spot, which a neighbouring wall can put outside the cell.
fn apply_authored_collisions(id: i32, def: i32) {
    pin_authored_wall_height(id, def);
    let clear_blocking = |quad_map_rays: bool| crate::level::clear_blocking(id, quad_map_rays);
    let clear_volume = || {
        let _ = spring::set_unit_collision_volume_data(
            id,
            spring::Float3::new(0.0, 0.0, 0.0),
            spring::Float3::new(0.0, 0.0, 0.0),
            0,
            0,
            0,
        );
    };

    // unit_collisions.lua
    let rule = collision(def);
    if rule.scenery_wall {
        clear_volume();
        clear_blocking(false);
        return;
    }
    // unit_plate.lua.  A lever keeps quad_map_rays so TraceScreenRay can still
    // find one to click; everything else loses its volume too.
    if rule.trigger {
        if def == defs().lever {
            clear_blocking(true);
        } else {
            clear_volume();
            clear_blocking(false);
        }
    }
}

#[derive(Default)]
pub struct LifecycleGadget {
    /// Whether the one-shot sweep over the map's own units has run.
    swept: core::cell::Cell<bool>,
}

impl LifecycleGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for LifecycleGadget {
    fn name(&self) -> &'static str {
        "Lifecycle"
    }

    /// Apply the authored collision rules to the units the map itself placed.
    ///
    /// UnitCreated does not reach them: they exist before this module loads.
    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        param("alive_wisps", 0.0);
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        if self.swept.replace(true) {
            return;
        }
        let Ok(units) = spring::get_all_units() else {
            return;
        };
        for id in units {
            if let Ok(def) = spring::get_unit_def_id(id) {
                apply_authored_collisions(id, def.0);
                unit_scripts::queue_unit_created(id, def.0);
            }
        }
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
        unit_scripts::queue_unit_created(id, def);
        let defs = defs();
        if def == defs.wisp {
            {
                let mut wisp = global.wisp.access_mut();
                wisp.id = id;
                wisp.pieces_dirty = true;
            }
            let _ = spring::set_unit_rules_param_f32(id, "selfMoveSpeedChange", 1.0, PUBLIC);
            global.pending.access_mut().piece_hides.push(id);
        }
        if defs.body_part(def).is_some() {
            global.wisp.access_mut().body_parts.push(id);
        }
        apply_authored_collisions(id, def);
        if def == defs.cage {
            global.pending.access_mut().cage_wisps.push(id);
        }
        if def == defs.npcwisp {
            let alive = {
                let mut world = global.world.access_mut();
                world.npc_wisps.push(id);
                world.npc_wisps.len() as f32
            };
            param("alive_wisps", alive);
            global.pending.access_mut().piece_hides.push(id);
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
        let alive = {
            let mut world = global.world.access_mut();
            world.npc_wisps.retain(|wisp| *wisp != id);
            world.npc_wisps.len() as f32
        };
        param("alive_wisps", alive);
    }
}
