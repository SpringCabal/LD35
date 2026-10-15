use core::cell::Cell;

use spring::{ResultExt, commands};
use spring_addons::rules::Gadget;

use crate::defs::{defs, params};
use crate::state::Global;

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

    let rule = params(def);
    if rule.scenery_wall {
        clear_volume();
        clear_blocking(false);
        return;
    }
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
pub struct CollisionsGadget {
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
