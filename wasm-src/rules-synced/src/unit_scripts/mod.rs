//! Parts Needed's unit scripts, implemented on top of the portable CUS API.
//!
//! The CUS instance registry lives here because a named unit-script call only
//! carries the CUS instance id, not the engine unit id.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use partsneeded_shared::{BodyPart, BodyParts};
use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{
    Angle, Axis, CusHandle, CusInstance, CusRegistry, InitCtx, Piece, PieceResolver,
    ScriptCapabilities, SignalMask, TaskDefinition, UnitCtx, UnitScript, UnitScriptCall,
    WreckLevel,
};

use crate::wisp::group_visible;

mod props;
mod wisp;

use props::{
    PropPieces, Props, cage_close, cage_open, gate_close, gate_open, lever_off, lever_on,
    plate_off, plate_on, props,
};
use wisp::{
    WispAnimationState, WispPieces, apply_wisp_idle_frame, start_wisp_movement, wisp_create,
    wisp_idle,
};

// Signals are per unit and gates never walk, so the two share a bit.
pub(crate) const SIG_GATE: SignalMask = SignalMask::new(1);
pub(crate) const SIG_WALK: SignalMask = SignalMask::new(1);
pub(crate) const SIG_FLOAT: SignalMask = SignalMask::new(4);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptKind {
    Wisp,
    Gate,
    Lever,
    Plate,
    Cage,
    Trigger,
}

fn script_kind_for_def(def: i32) -> Option<ScriptKind> {
    let defs = crate::defs::defs();
    if def < 0 {
        None
    } else if def == defs.wisp || def == defs.npcwisp {
        Some(ScriptKind::Wisp)
    } else if def == defs.gate {
        Some(ScriptKind::Gate)
    } else if def == defs.lever {
        Some(ScriptKind::Lever)
    } else if def == defs.plate {
        Some(ScriptKind::Plate)
    } else if def == defs.cage {
        Some(ScriptKind::Cage)
    } else if def == defs.light || defs.body_part(def).is_some() {
        Some(ScriptKind::Trigger)
    } else {
        None
    }
}

fn capabilities(kind: ScriptKind) -> ScriptCapabilities {
    let mut bits = ScriptCapabilities::KILLED;
    match kind {
        ScriptKind::Wisp => {
            bits |= ScriptCapabilities::CREATE
                | ScriptCapabilities::START_MOVING
                | ScriptCapabilities::STOP_MOVING;
        }
        ScriptKind::Gate | ScriptKind::Lever | ScriptKind::Plate | ScriptKind::Cage => {
            bits |= ScriptCapabilities::CREATE
                | ScriptCapabilities::ACTIVATE
                | ScriptCapabilities::DEACTIVATE;
        }
        ScriptKind::Trigger => {}
    }
    ScriptCapabilities::new(bits)
}

/// Instance ids are derived from unit ids and are unique within this module.
fn instance_id(unit: UnitId) -> u32 {
    0x504e_0000u32.wrapping_add(unit.0 as u32)
}

struct EnginePieces {
    map: Vec<spring::owned::units_pieces::PieceMapEntry>,
    root: Piece,
}

impl PieceResolver for EnginePieces {
    fn piece(&self, name: &str) -> Piece {
        if name == "__partsneeded_model_root" {
            return self.root;
        }
        self.map
            .iter()
            .find(|entry| entry.name == name)
            // The public piece map follows Lua's one-based numbering. CUS
            // follows CUnitScript's zero-based numbering.
            .map_or(Piece::INVALID, |entry| Piece(entry.piece_num - 1))
    }
}

struct PartsUnitScript {
    kind: Option<ScriptKind>,
    pieces: WispPieces,
    props: Props,
    animation: Rc<RefCell<WispAnimationState>>,
}

impl UnitScript for PartsUnitScript {
    fn new(ctx: &mut InitCtx<'_>) -> Self {
        let pieces = WispPieces::from_init(ctx);
        Self {
            kind: None,
            pieces,
            props: Rc::new(RefCell::new(PropPieces::from_init(ctx))),
            animation: Rc::new(RefCell::new(WispAnimationState {
                pieces,
                parts: None,
                moving: false,
            })),
        }
    }

    fn create(&mut self, ctx: &UnitCtx) {
        match self.kind {
            Some(ScriptKind::Wisp) => {
                // Lua's script.Create applies the first idle keyframe before
                // creating any sleeping animation thread.  Do the same
                // synchronously; putting this in a task shifts the whole
                // startup pose by one simulation frame.
                apply_wisp_idle_frame(ctx, self.pieces);
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-wisp-create",
                    Rc::clone(&self.animation),
                    wisp_create,
                ));
            }
            Some(ScriptKind::Lever) => {
                ctx.turn_now(props(&self.props).lever, Axis::X, Angle::radians(0.6));
            }
            _ => {}
        }
    }

    fn start_moving(&mut self, ctx: &UnitCtx, _reversing: bool) {
        if self.kind != Some(ScriptKind::Wisp) {
            return;
        }
        let already_moving = {
            let mut state = self.animation.borrow_mut();
            let already_moving = state.moving;
            state.moving = true;
            already_moving
        };
        if !already_moving {
            start_wisp_movement(ctx, Rc::clone(&self.animation));
        }
    }

    fn stop_moving(&mut self, ctx: &UnitCtx) {
        if self.kind != Some(ScriptKind::Wisp) {
            return;
        }
        self.animation.borrow_mut().moving = false;
        ctx.signal(SIG_WALK);
        ctx.spawn(TaskDefinition::with_state(
            "partsneeded-wisp-stop",
            Rc::clone(&self.animation),
            wisp_idle,
        ));
    }

    fn activate(&mut self, ctx: &UnitCtx) {
        match self.kind {
            Some(ScriptKind::Gate) => {
                ctx.signal(SIG_GATE);
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-gate-open",
                    Rc::clone(&self.props),
                    gate_open,
                ));
            }
            Some(ScriptKind::Lever) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-lever-on",
                    Rc::clone(&self.props),
                    lever_on,
                ));
            }
            Some(ScriptKind::Plate) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-plate-on",
                    Rc::clone(&self.props),
                    plate_on,
                ));
            }
            Some(ScriptKind::Cage) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-cage-open",
                    Rc::clone(&self.props),
                    cage_open,
                ));
            }
            _ => {}
        }
    }

    fn deactivate(&mut self, ctx: &UnitCtx) {
        match self.kind {
            Some(ScriptKind::Gate) => {
                ctx.signal(SIG_GATE);
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-gate-close",
                    Rc::clone(&self.props),
                    gate_close,
                ));
            }
            Some(ScriptKind::Lever) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-lever-off",
                    Rc::clone(&self.props),
                    lever_off,
                ));
            }
            Some(ScriptKind::Plate) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-plate-off",
                    Rc::clone(&self.props),
                    plate_off,
                ));
            }
            Some(ScriptKind::Cage) => {
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-cage-close",
                    Rc::clone(&self.props),
                    cage_close,
                ));
            }
            _ => {}
        }
    }

    fn killed(&mut self, _ctx: &UnitCtx, _recent_damage: f32, _max_health: f32) -> WreckLevel {
        match self.kind {
            Some(ScriptKind::Wisp | ScriptKind::Trigger) => WreckLevel::None,
            _ => WreckLevel::One,
        }
    }
}

impl PartsUnitScript {
    /// Show the pieces of the collected body parts (`None`: a spirit, fully
    /// hidden), and switch to the gait the limbs allow.
    fn set_piece_mask(&mut self, ctx: &UnitCtx, parts: Option<BodyParts>) {
        let p = self.pieces;
        let old_parts = self.animation.borrow().parts;
        for piece in p.all {
            ctx.hide(piece);
        }
        self.animation.borrow_mut().parts = parts;
        let Some(parts) = parts else {
            return;
        };
        for (piece, group) in p.groups {
            if group_visible(group, Some(parts)) {
                ctx.show(piece);
            }
        }

        // SetPieceVisible() in the Lua script calls updateMovement() whenever
        // the arms or legs group changes: a moving wisp switches to the gait
        // its limbs allow (walk with legs, float_arms "swim" with arms only),
        // a standing one to idle.  Without this, a wisp that gains arms while
        // already moving keeps its old animation, and the first spirit ->
        // corporal switch leaves the body frozen at its spirit-form pose.
        let limbs = |parts: BodyParts| {
            (
                parts.contains(BodyPart::Arms),
                parts.contains(BodyPart::Legs),
            )
        };
        let limbs_changed = old_parts.is_none_or(|old| limbs(old) != limbs(parts));
        if self.kind == Some(ScriptKind::Wisp) && limbs_changed && limbs(parts) != (false, false) {
            if self.animation.borrow().moving {
                start_wisp_movement(ctx, Rc::clone(&self.animation));
            } else {
                ctx.signal(SIG_WALK);
                ctx.spawn(TaskDefinition::with_state(
                    "partsneeded-wisp-mask-idle",
                    Rc::clone(&self.animation),
                    wisp_idle,
                ));
            }
        }
    }
}

thread_local! {
    static PENDING_UNITS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
    static PENDING_MASKS: RefCell<Vec<(i32, Option<BodyParts>)>> = const { RefCell::new(Vec::new()) };
}

/// Queue a wisp piece mask (wisp.lua's SetPieceVisible groups) for the next
/// Core-CUS tick.  The rules run inside this same module, so routing the call
/// through the engine (`call_unit_script`) is a nested call into a module that
/// is already running a gadget callin; the engine rejects it (ApiError 1) and
/// the mask never arrived, leaving the wisp's animation state in spirit form.
pub fn queue_piece_mask(unit: i32, parts: Option<BodyParts>) {
    if unit >= 0 {
        PENDING_MASKS.with(|pending| pending.borrow_mut().push((unit, parts)));
    }
}

/// Queue both map-authored units and units spawned by the Rust rules. Attach
/// itself is deferred until the Core-CUS tick so the engine never sees a
/// nested attach while a gadget callin is active.
pub fn queue_unit_created(unit: i32, def: i32) {
    if unit >= 0 {
        PENDING_UNITS.with(|pending| pending.borrow_mut().push((unit, def)));
    }
}

#[derive(Default)]
pub struct PartsCusModule {
    registry: CusRegistry<PartsUnitScript>,
    handles: BTreeMap<u32, CusHandle>,
}

impl PartsCusModule {
    fn attach_pending(&mut self) {
        let pending = PENDING_UNITS.with(|units| core::mem::take(&mut *units.borrow_mut()));
        for (unit_id, def) in pending {
            let Some(kind) = script_kind_for_def(def) else {
                continue;
            };
            let unit = UnitId(unit_id);
            let instance = instance_id(unit);
            if self.handles.contains_key(&instance) {
                continue;
            }
            let Ok(pieces) = spring::get_unit_piece_map(unit_id) else {
                continue;
            };
            let root = spring::get_unit_root_piece(unit_id)
                .ok()
                // The public UnitsPieces APIs are one-based, while CUS
                // piece ids are zero-based like CUnitScript::pieces.
                .and_then(|piece| piece.checked_sub(1))
                .map(Piece)
                .unwrap_or(Piece::INVALID);
            let resolver = EnginePieces { map: pieces, root };
            let host = WasmCus::new(unit, instance);
            let engine = Rc::new(RefCell::new(host.engine()));
            let instance_state = CusInstance::construct(unit, &resolver, engine);
            let handle = self.registry.attach(instance_state);
            let _ = self.registry.with(handle, |instance| {
                instance.with_state(|script| script.kind = Some(kind));
            });
            if WasmCus::attach(unit, instance, capabilities(kind)).is_ok() {
                self.handles.insert(instance, handle);
            } else {
                let _ = self.registry.detach(handle);
            }
        }
    }

    fn apply_pending_masks(&mut self) {
        let pending = PENDING_MASKS.with(|masks| core::mem::take(&mut *masks.borrow_mut()));
        for (unit_id, parts) in pending {
            let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
                continue;
            };
            self.sync_instance_frame(handle);
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.set_piece_mask(&ctx, parts));
            });
        }
    }

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
        // Engine movement callbacks are delivered through cus_invoke rather
        // than the named-call ABI. They can arrive after this module's normal
        // per-frame tick, so advance the scheduler boundary before invoking a
        // callback that starts or cancels tasks.
        if matches!(
            call,
            UnitScriptCall::RawCall | UnitScriptCall::StartMoving | UnitScriptCall::StopMoving
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

    fn cus_call_named(
        &mut self,
        instance_id: u32,
        function_name: &str,
        arguments: &[f32],
        _return_values: &mut [f32],
        found: &mut bool,
    ) -> Option<usize> {
        let call = match function_name {
            "SetWasmPieceMask" => UnitScriptCall::RawCall,
            "StartMoving" => UnitScriptCall::StartMoving,
            "StopMoving" => UnitScriptCall::StopMoving,
            _ => return None,
        };
        *found = true;
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return Some(0);
        };
        // Core-Wasm named calls can arrive from a synced gadget before the
        // host's per-frame CUS tick. Advance this instance boundary first so
        // Sleep/next-frame deadlines are relative to the actual game frame,
        // not the previous frame at which the module happened to be ticked.
        self.sync_instance_frame(handle);
        if call == UnitScriptCall::RawCall {
            // The argument is the part bits, or negative for a spirit.
            let mask = arguments.first().copied().unwrap_or(-1.0);
            let parts = (mask >= 0.0).then(|| BodyParts::from_bits(mask as u8));
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.set_piece_mask(&ctx, parts));
            });
        } else {
            let _ = self.registry.with(handle, |instance| {
                let mut result = spring::cus::UnitScriptCallResult::default();
                let _ = instance.invoke(call, &[], &[], &mut result);
            });
        }
        Some(0)
    }

    fn cus_tick(&mut self, frame: u32) {
        self.attach_pending();
        self.apply_pending_masks();
        self.registry.tick(frame as u64);
    }

    fn cus_detach(&mut self, instance_id: u32) {
        if let Some(handle) = self.handles.remove(&instance_id) {
            let _ = self.registry.detach(handle);
        }
    }
}
