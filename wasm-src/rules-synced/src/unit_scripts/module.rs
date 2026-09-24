//! The Core-CUS module: the registry of attached scripts and the engine's
//! entry points into them.
//!
//! The rules reach it directly through `crate::with_cus_module` to attach
//! scripts and apply piece masks.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use partsneeded_shared::BodyParts;
use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{CusHandle, CusInstance, CusRegistry, Piece, PieceResolver, UnitScriptCall};

use super::{PartsUnitScript, capabilities, script_kind_for_def};
use crate::log::SECTION;

/// Instance ids are derived from unit ids and are unique within this module.
fn instance_id(unit: UnitId) -> u32 {
    0x504e_0000u32.wrapping_add(unit.0 as u32)
}

struct EnginePieces {
    map: Vec<spring::owned::units_pieces::PieceMapEntry>,
}

impl PieceResolver for EnginePieces {
    fn piece(&self, name: &str) -> Piece {
        self.map
            .iter()
            .find(|entry| entry.name == name)
            // The public piece map follows Lua's one-based numbering. CUS
            // follows CUnitScript's zero-based numbering.
            .map_or(Piece::INVALID, |entry| Piece(entry.piece_num - 1))
    }
}

#[derive(Default)]
pub struct PartsCusModule {
    registry: CusRegistry<PartsUnitScript>,
    handles: BTreeMap<u32, CusHandle>,
}

impl PartsCusModule {
    /// Give a new unit its script, if its def has one.  The engine runs the
    /// script's Create once the callin that attached it has returned.
    pub fn attach(&mut self, unit_id: i32, def: i32) {
        let Some(kind) = script_kind_for_def(def) else {
            return;
        };
        let unit = UnitId(unit_id);
        let instance = instance_id(unit);
        if self.handles.contains_key(&instance) {
            return;
        }
        let map = match spring::get_unit_piece_map(unit_id) {
            Ok(map) => map,
            Err(error) => {
                spring::log_warning!(
                    SECTION,
                    "no script for unit {unit_id}: piece map unavailable: {error:?}"
                );
                return;
            }
        };
        let host = WasmCus::new(unit, instance);
        let engine = Rc::new(RefCell::new(host.engine()));
        let instance_state = CusInstance::construct(unit, &EnginePieces { map }, engine);
        let handle = self.registry.attach(instance_state);
        let _ = self.registry.with(handle, |instance| {
            instance.with_state(|script| script.kind = Some(kind));
        });
        if let Err(error) = WasmCus::attach(unit, instance, capabilities(kind)) {
            let _ = self.registry.detach(handle);
            spring::log_warning!(
                SECTION,
                "attaching the {kind:?} script to unit {unit_id} failed: {error:?}"
            );
            return;
        }
        self.handles.insert(instance, handle);
        // Start the new instance's clock at the current frame, so the sleeps
        // in its Create count from the frame the unit was created on.
        self.sync_instance_frame(handle);
    }

    /// Apply a wisp piece mask (wisp.lua's SetPieceVisible groups); `None` is
    /// a spirit, with every piece hidden.
    pub fn set_piece_mask(&mut self, unit_id: i32, parts: Option<BodyParts>) {
        let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
            spring::log_warning!(SECTION, "piece mask for unit {unit_id}: it has no script");
            return;
        };
        self.sync_instance_frame(handle);
        let _ = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| script.set_piece_mask(&ctx, parts));
        });
    }

    /// Rules calls can arrive before this frame's CUS tick.  Advance the
    /// instance to the current frame first, so the sleeps and tasks they start
    /// count from the actual game frame.
    fn sync_instance_frame(&mut self, handle: CusHandle) {
        let frame = spring::game::frame().map_or(0, u64::from);
        let _ = self.registry.with(handle, |instance| instance.tick(frame));
    }
}

impl CoreCusModule for PartsCusModule {
    fn cus_invoke(
        &mut self,
        instance_id: u32,
        call: u32,
        float_arguments: &[f32],
        integer_arguments: &[i32],
        result: &mut CoreCusCallResult<'_>,
    ) -> bool {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return false;
        };
        let Some(call) = UnitScriptCall::from_u32(call) else {
            return false;
        };
        // Movement callbacks can arrive after this module's per-frame tick,
        // so advance the instance before a callback that starts or cancels
        // tasks.
        if matches!(
            call,
            UnitScriptCall::StartMoving | UnitScriptCall::StopMoving
        ) {
            self.sync_instance_frame(handle);
        }
        let mut call_result = spring::cus::UnitScriptCallResult::default();
        let handled = self
            .registry
            .with(handle, |instance| {
                instance.invoke(call, float_arguments, integer_arguments, &mut call_result)
            })
            .unwrap_or(false);
        if !handled {
            return false;
        }
        result.int_value = call_result.int_value;
        result.float_value = call_result.float_value;
        result.bool_value = call_result.bool_value;
        result.complete = call_result.complete;
        result.int_count = call_result.int_values.len().min(result.int_values.len());
        result.int_values[..result.int_count]
            .copy_from_slice(&call_result.int_values[..result.int_count]);
        true
    }

    fn cus_tick(&mut self, frame: u32) {
        self.registry.tick(frame as u64);
    }

    fn cus_detach(&mut self, instance_id: u32) {
        if let Some(handle) = self.handles.remove(&instance_id) {
            let _ = self.registry.detach(handle);
        }
    }
}
