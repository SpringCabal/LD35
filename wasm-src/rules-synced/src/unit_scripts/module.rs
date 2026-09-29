//! The Core-CUS module: the engine's `CusHost` over Parts Needed's scripts.
//!
//! The rules reach it through [`with_host`] to attach scripts and apply
//! piece masks.

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

/// Apply a wisp piece mask (wisp.lua's SetPieceVisible groups); `None` is a
/// spirit, with every piece hidden.
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

/// Run `f` on the module's scripts now.
///
/// The engine sets the module up when it loads it, before any callin, and
/// nothing holds it when the rules run: the scripts make no engine calls that
/// come back to the rules, and the rules' own such calls are commands, applied
/// after they return.
pub fn with_host(f: impl FnOnce(&mut PartsCusModule)) {
    if crate::with_cus_module(f).is_none() {
        spring::log_warning!(SECTION, "the unit-script module is busy");
    }
}
