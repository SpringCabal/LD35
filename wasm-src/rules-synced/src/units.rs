use alloc::vec::Vec;

pub type Removal = (i32, bool, i32);

pub fn destroy(_spring: &spring::Spring<'_, spring::RulesSynced>, units: Vec<Removal>) {
    for (id, reclaimed, attacker) in units {
        crate::with_addon_context(|ctx| {
            ctx.commands().destroy_unit(
                id,
                spring::unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed,
                    attacker_id: attacker,
                    recycle_id: false,
                },
            )
        })
        .then(move |spring, result| {
            crate::log::check(spring, "destroy", id, result);
        });
    }
}
