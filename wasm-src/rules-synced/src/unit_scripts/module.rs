use partsneeded_shared::BodyParts;
use spring::UnitId;
use spring::cus::host::{CusHost, UnitDefScript};
use spring::cus::{InitCtx, ScriptCapabilities};

use super::{PartsUnitScript, capabilities, script_kind_for_def};
use crate::log::SECTION;

pub type PartsCusModule = CusHost<PartsUnitScript>;

impl UnitDefScript for PartsUnitScript {
    fn capabilities(def: i32) -> Option<ScriptCapabilities> {
        script_kind_for_def(def).map(capabilities)
    }

    fn for_def(def: i32, ctx: &mut InitCtx<'_>, _shared: &mut ()) -> Self {
        PartsUnitScript::for_kind(script_kind_for_def(def), ctx)
    }
}

pub fn set_piece_mask(host: &mut PartsCusModule, unit_id: i32, parts: Option<BodyParts>) {
    if host
        .with_script(UnitId(unit_id), |ctx, script| {
            script.set_piece_mask(ctx, parts)
        })
        .is_none()
    {
        spring::log_warning!(SECTION, "piece mask for unit {unit_id}: it has no script");
    }
}

pub fn with_host(f: impl FnOnce(&mut PartsCusModule)) {
    if crate::with_cus_module(f).is_none() {
        spring::log_warning!(SECTION, "the unit-script module is busy");
    }
}
