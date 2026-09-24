//! Unit removal, shared by the gadgets that retire units.

use alloc::vec::Vec;

/// A unit to destroy: its id, whether it is reclaimed (no death animation),
/// and the attacker to credit (-1 for none).
pub type Removal = (i32, bool, i32);

/// Destroy `units` from a delayed action: UnitDestroyed is raised
/// synchronously and needs the state the calling gadget may hold.
pub fn destroy_later(
    ctx: &spring_addons::AddonContext<'_, crate::state::Global>,
    units: Vec<Removal>,
) {
    if units.is_empty() {
        return;
    }
    ctx.delay(move |_| {
        for (id, reclaimed, attacker) in units {
            let result = spring::destroy_unit(
                id,
                spring::rules_synced::unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed,
                    attacker_id: attacker,
                    recycle_id: false,
                },
            );
            crate::log::check("destroy", id, result);
        }
    });
}
