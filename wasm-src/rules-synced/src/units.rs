//! Unit removal, shared by the gadgets that retire units.

use alloc::vec::Vec;

use spring::commands;

/// A unit to destroy: its id, whether it is reclaimed (no death animation),
/// and the attacker to credit (-1 for none).
pub type Removal = (i32, bool, i32);

/// Destroy `units` once the current callin has returned.
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
