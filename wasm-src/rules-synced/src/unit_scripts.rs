use alloc::rc::Rc;
use core::cell::RefCell;

use partsneeded_shared::{BodyPart, BodyParts};
use spring::cus::{
    Angle, Axis, InitCtx, ScriptCapabilities, SignalMask, TaskDefinition, UnitCtx, UnitScript,
    WreckLevel,
};

use crate::gadgets::wisp::behavior::group_visible;

mod module;
mod props;
mod wisp;

pub use module::{PartsCusModule, set_piece_mask, with_host};

use props::{
    PropPieces, Props, cage_close, cage_open, gate_close, gate_open, lever_off, lever_on,
    plate_off, plate_on, props,
};
use wisp::{
    WispAnimationState, WispPieces, apply_wisp_idle_frame, start_wisp_movement, wisp_create,
    wisp_idle,
};

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

fn script_kind_for_def(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    def: i32,
) -> Option<ScriptKind> {
    let defs = crate::defs::defs(spring);
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

pub struct PartsUnitScript {
    kind: Option<ScriptKind>,
    pieces: WispPieces,
    props: Props,
    animation: Rc<RefCell<WispAnimationState>>,
}

impl PartsUnitScript {
    fn for_kind(kind: Option<ScriptKind>, ctx: &mut InitCtx<'_>) -> Self {
        let pieces = WispPieces::from_init(ctx);
        Self {
            kind,
            pieces,
            props: Rc::new(RefCell::new(PropPieces::from_init(ctx))),
            animation: Rc::new(RefCell::new(WispAnimationState {
                pieces,
                parts: None,
                moving: false,
            })),
        }
    }
}

impl UnitScript for PartsUnitScript {
    fn create(&mut self, ctx: &UnitCtx) {
        match self.kind {
            Some(ScriptKind::Wisp) => {
                for piece in self.pieces.all {
                    ctx.hide(piece);
                }
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
