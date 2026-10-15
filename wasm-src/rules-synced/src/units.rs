use alloc::vec::Vec;

use spring::commands;

pub type Removal = (i32, bool, i32);

pub fn destroy(units: Vec<Removal>) {
    for (id, reclaimed, attacker) in units {
        commands::destroy_unit(
            id,
            spring::rules_synced::unit_control::DestroyUnitOptions {
                selfd: false,
                reclaimed,
                attacker_id: attacker,
                recycle_id: false,
            },
        )
        .then(move |result| {
            crate::log::check("destroy", id, result);
        });
    }
}
