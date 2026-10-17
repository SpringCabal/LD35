use core::cell::Cell;

use spring::ResultExt;
use spring_addons::rules::Gadget;

use crate::defs::{defs, params};
use crate::state::Global;

fn pin_authored_wall_height(spring: &spring::Spring<'_, spring::RulesSynced>, id: i32, def: i32) {
    if !params(spring, def).pinned_wall {
        return;
    }
    let Ok(position) = spring
        .units_info()
        .get_unit_position(id, Default::default())
    else {
        return;
    };
    spring
        .move_ctrl()
        .move_ctrl(id, true)
        .log_err_once(spring, "pinning a wall: MoveCtrl");
    spring
        .move_ctrl()
        .set_move_ctrl_gravity(id, 0.0)
        .log_err_once(spring, "pinning a wall: gravity");
    crate::with_addon_context(|ctx| {
        ctx.commands().set_unit_position(
            id,
            spring::Float3::new(position.x, crate::level::FLOOR_Y, position.z),
        )
    })
    .then(|spring, result| {
        result.log_err_once(spring, "pinning a wall: floor height");
    });
}

fn apply_authored_collisions(spring: &spring::Spring<'_, spring::RulesSynced>, id: i32, def: i32) {
    pin_authored_wall_height(spring, id, def);
    let clear_blocking =
        |quad_map_rays: bool| crate::level::clear_blocking(spring, id, quad_map_rays);
    let clear_volume = || {
        spring
            .unit_control()
            .set_unit_collision_volume_data(
                id,
                spring::Float3::new(0.0, 0.0, 0.0),
                spring::Float3::new(0.0, 0.0, 0.0),
                0,
                0,
                0,
            )
            .log_err_once(spring, "clearing a collision volume");
    };

    let rule = params(spring, def);
    if rule.scenery_wall {
        clear_volume();
        clear_blocking(false);
        return;
    }
    if rule.trigger {
        if def == defs(spring).lever {
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
        let spring = _ctx.spring();
        if self.swept.replace(true) {
            return;
        }
        for id in spring.units().get_all_units().unwrap_or_default() {
            if let Ok(def) = spring.units_info().get_unit_def_id(id) {
                apply_authored_collisions(spring, id, def);
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
        let spring = _ctx.spring();
        apply_authored_collisions(spring, id, def);
    }
}
