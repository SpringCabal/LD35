use partsneeded_shared::BodyParts;
use spring::cus::host::{CusHost, UnitDefScript};
use spring::cus::{InitCtx, ScriptCapabilities};
use spring::{Result, ResultExt, RulesSynced, Spring, UnitId};

use super::{PartsUnitScript, capabilities, script_kind_for_def};
use crate::log::SECTION;

pub type PartsCusModule = CusHost<PartsUnitScript>;

impl UnitDefScript for PartsUnitScript {
    fn capabilities(spring: &Spring<'_, RulesSynced>, def: i32) -> Option<ScriptCapabilities> {
        script_kind_for_def(spring, def).map(capabilities)
    }
    fn for_def(
        spring: &Spring<'_, RulesSynced>,
        def: i32,
        ctx: &mut InitCtx<'_>,
        _shared: &mut (),
    ) -> Self {
        PartsUnitScript::for_kind(script_kind_for_def(spring, def), ctx)
    }
}

pub fn set_piece_mask(
    host: &PartsCusModule,
    spring: &Spring<'_, RulesSynced>,
    unit_id: i32,
    parts: Option<BodyParts>,
) -> Result<()> {
    if host
        .with_script(spring, UnitId(unit_id), |ctx, script| {
            script.set_piece_mask(ctx, parts)
        })?
        .is_none()
    {
        spring::log_warning!(
            spring,
            SECTION,
            "piece mask for unit {unit_id}: it has no script"
        );
    }
    Ok(())
}

pub fn with_host(
    spring: &Spring<'_, RulesSynced>,
    f: impl FnOnce(&PartsCusModule, &Spring<'_, RulesSynced>) -> Result<()>,
) {
    crate::with_cus_module(|host, spring| f(host, spring))
        .log_err_once(spring, "Parts Needed unit scripts");
}
