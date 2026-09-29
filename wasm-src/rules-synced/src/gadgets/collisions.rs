use core::cell::Cell;

use spring::{ResultExt, commands};
use spring_addons::rules::Gadget;

use crate::defs::{defs, params};
use crate::state::Global;

/// Keep map-authored walls at the level's explicit floor height.  A map unit
/// can predate the WASM rules module, so it does not necessarily pass through
/// level::create(), where the same MoveCtrl pin is applied.
fn pin_authored_wall_height(id: i32, def: i32) {
    if !params(def).pinned_wall {
        return;
    }
    let Ok(position) = spring::get_unit_position(id, false, false) else {
        return;
    };
    spring::move_ctrl::move_ctrl(id, true).log_err_once("pinning a wall: MoveCtrl");
    spring::move_ctrl::set_move_ctrl_gravity(id, 0.0).log_err_once("pinning a wall: gravity");
    commands::set_unit_position(
        id,
        spring::Float3::new(position.x, crate::level::FLOOR_Y, position.z),
    )
    .then(|result| {
        result.log_err_once("pinning a wall: floor height");
    });
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
        spring::set_unit_collision_volume_data(
            id,
            spring::Float3::new(0.0, 0.0, 0.0),
            spring::Float3::new(0.0, 0.0, 0.0),
            0,
            0,
            0,
        )
        .log_err_once("clearing a collision volume");
    };

    // unit_collisions.lua
    let rule = params(def);
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

/// The authored collision rules: unit_collisions.lua and unit_plate.lua.
#[derive(Default)]
pub struct CollisionsGadget {
    /// Whether the one-shot sweep over the map's own units has run.
    swept: Cell<bool>,
}

impl CollisionsGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for CollisionsGadget {
    fn name(&self) -> &'static str {
        "Collisions"
    }

    /// Apply the rules to the units the map itself placed.
    ///
    /// UnitCreated does not reach them: they exist before this module loads.
    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        if self.swept.replace(true) {
            return;
        }
        for id in spring::get_all_units().unwrap_or_default() {
            if let Ok(def) = spring::get_unit_def_id(id) {
                apply_authored_collisions(id, def.0);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        id: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        apply_authored_collisions(id, def);
    }
}
