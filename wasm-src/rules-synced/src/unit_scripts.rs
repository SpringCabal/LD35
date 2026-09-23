//! Parts Needed's unit scripts, implemented on top of the portable CUS API.
//!
//! The CUS instance registry lives here because a named unit-script call only
//! carries the CUS instance id, not the engine unit id.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{
    Angle, Axis, CusHandle, CusInstance, CusRegistry, Duration, InitCtx, Piece, PieceResolver,
    ScriptCapabilities, SignalMask, TaskDefinition, UnitCtx, UnitScript, UnitScriptCall,
    WreckLevel,
};

// Signals are per unit and gates never walk, so the two share a bit.
const SIG_GATE: SignalMask = SignalMask::new(1);
const SIG_WALK: SignalMask = SignalMask::new(1);
const SIG_FLOAT: SignalMask = SignalMask::new(4);

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
    let matches = |name: &str| spring::get_unit_def_id_by_name(name).ok() == Some(def);
    if matches("wisp") || matches("npcwisp") {
        Some(ScriptKind::Wisp)
    } else if matches("gatesmoth") {
        Some(ScriptKind::Gate)
    } else if matches("lever") {
        Some(ScriptKind::Lever)
    } else if matches("plate") {
        Some(ScriptKind::Plate)
    } else if matches("cage") {
        Some(ScriptKind::Cage)
    } else if matches("eyes")
        || matches("ears")
        || matches("nose")
        || matches("mouth")
        || matches("arms")
        || matches("legs")
        || matches("light")
    {
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

#[derive(Clone, Copy, Default)]
struct WispPieces {
    all: [Piece; 22],
    arms: [Piece; 6],
    legs: [Piece; 6],
    body: Piece,
    head: Piece,
    eyes: Piece,
    ears: Piece,
    nose: Piece,
    mouth: Piece,
    thigh_l: Piece,
    thigh_r: Piece,
    crus_l: Piece,
    crus_r: Piece,
    foot_l: Piece,
    foot_r: Piece,
    arm_l: Piece,
    arm_r: Piece,
    forearm_l: Piece,
    forearm_r: Piece,
    hair1: Piece,
    hair2: Piece,
    hair3: Piece,
    hair4: Piece,
    hand_l: Piece,
    hand_r: Piece,
}

impl WispPieces {
    fn from_init(ctx: &InitCtx<'_>) -> Self {
        let p = |name: &str| ctx.piece(name);
        let arm_l = p("ArmL");
        let arm_r = p("ArmR");
        let crus_l = p("CrusL");
        let crus_r = p("CrusR");
        let foot_l = p("FootL");
        let foot_r = p("FootR");
        let forearm_l = p("ForeArmL");
        let forearm_r = p("ForeArmR");
        let hand_l = p("HandL");
        let hand_r = p("HandR");
        let thigh_l = p("ThighL");
        let thigh_r = p("ThighR");
        let hair1 = p("Hair1");
        let hair2 = p("Hair2");
        let hair3 = p("Hair3");
        let hair4 = p("Hair4");
        let head = p("Head");
        let eyes = p("Eyes");
        let ears = p("Ears");
        let body = p("Body");
        let mouth = p("Mouth");
        let nose = p("Nose");
        Self {
            all: [
                arm_l, arm_r, crus_l, crus_r, foot_l, foot_r, forearm_l, forearm_r, hair1, hair2,
                hair3, hair4, hand_r, head, eyes, ears, hand_l, body, mouth, nose, thigh_l,
                thigh_r,
            ],
            arms: [arm_l, arm_r, forearm_l, forearm_r, hand_l, hand_r],
            legs: [thigh_l, thigh_r, crus_l, crus_r, foot_l, foot_r],
            body,
            head,
            eyes,
            ears,
            nose,
            mouth,
            thigh_l,
            thigh_r,
            crus_l,
            crus_r,
            foot_l,
            foot_r,
            arm_l,
            arm_r,
            forearm_l,
            forearm_r,
            hair1,
            hair2,
            hair3,
            hair4,
            hand_l,
            hand_r,
        }
    }
}

struct WispAnimationState {
    pieces: WispPieces,
    mask: i32,
    moving: bool,
}

struct PartsUnitScript {
    kind: Option<ScriptKind>,
    pieces: WispPieces,
    animation: Rc<RefCell<WispAnimationState>>,
}

impl UnitScript for PartsUnitScript {
    fn new(ctx: &mut InitCtx<'_>) -> Self {
        let pieces = WispPieces::from_init(ctx);
        Self {
            kind: None,
            pieces,
            animation: Rc::new(RefCell::new(WispAnimationState {
                pieces,
                mask: -1,
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
                ctx.turn_now(
                    unit_piece(ctx.unit(), "Base_001"),
                    Axis::X,
                    Angle::radians(0.6),
                );
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
                ctx.spawn(TaskDefinition::new("partsneeded-gate-open", gate_open));
            }
            Some(ScriptKind::Lever) => {
                ctx.spawn(TaskDefinition::new("partsneeded-lever-on", lever_on));
            }
            Some(ScriptKind::Plate) => {
                ctx.spawn(TaskDefinition::new("partsneeded-plate-on", plate_on));
            }
            Some(ScriptKind::Cage) => {
                ctx.spawn(TaskDefinition::new("partsneeded-cage-open", cage_open));
            }
            _ => {}
        }
    }

    fn deactivate(&mut self, ctx: &UnitCtx) {
        match self.kind {
            Some(ScriptKind::Gate) => {
                ctx.signal(SIG_GATE);
                ctx.spawn(TaskDefinition::new("partsneeded-gate-close", gate_close));
            }
            Some(ScriptKind::Lever) => {
                ctx.spawn(TaskDefinition::new("partsneeded-lever-off", lever_off));
            }
            Some(ScriptKind::Plate) => {
                ctx.spawn(TaskDefinition::new("partsneeded-plate-off", plate_off));
            }
            Some(ScriptKind::Cage) => {
                ctx.spawn(TaskDefinition::new("partsneeded-cage-close", cage_close));
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

fn turn(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if speed == 0.0 {
        ctx.turn_now(piece, axis, Angle::radians(target));
    } else {
        ctx.turn(
            piece,
            axis,
            Angle::radians(target),
            spring::cus::AngularSpeed::radians_per_second(speed),
        );
    }
}

fn move_piece(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if speed == 0.0 {
        ctx.move_now(piece, axis, target);
    } else {
        ctx.move_piece(piece, axis, target, speed);
    }
}

#[derive(Clone, Copy)]
enum WispAnimationPiece {
    Nose,
    Hair1,
    Head,
    Eyes,
    Ears,
    Hair3,
    Mouth,
    Hair2,
    Hair4,
    ArmL,
    ArmR,
    ForeArmL,
    ForeArmR,
    HandL,
    HandR,
    ThighL,
    ThighR,
    CrusL,
    CrusR,
    FootL,
    FootR,
}

#[derive(Clone, Copy)]
struct TurnCommand {
    piece: WispAnimationPiece,
    axis: Axis,
    target: f32,
    speed: f32,
}

fn animation_piece(p: WispPieces, piece: WispAnimationPiece) -> Piece {
    match piece {
        WispAnimationPiece::Nose => p.nose,
        WispAnimationPiece::Hair1 => p.hair1,
        WispAnimationPiece::Head => p.head,
        WispAnimationPiece::Eyes => p.eyes,
        WispAnimationPiece::Ears => p.ears,
        WispAnimationPiece::Hair3 => p.hair3,
        WispAnimationPiece::Mouth => p.mouth,
        WispAnimationPiece::Hair2 => p.hair2,
        WispAnimationPiece::Hair4 => p.hair4,
        WispAnimationPiece::ArmL => p.arm_l,
        WispAnimationPiece::ArmR => p.arm_r,
        WispAnimationPiece::ForeArmL => p.forearm_l,
        WispAnimationPiece::ForeArmR => p.forearm_r,
        WispAnimationPiece::HandL => p.hand_l,
        WispAnimationPiece::HandR => p.hand_r,
        WispAnimationPiece::ThighL => p.thigh_l,
        WispAnimationPiece::ThighR => p.thigh_r,
        WispAnimationPiece::CrusL => p.crus_l,
        WispAnimationPiece::CrusR => p.crus_r,
        WispAnimationPiece::FootL => p.foot_l,
        WispAnimationPiece::FootR => p.foot_r,
    }
}

fn apply_turn_commands(ctx: &UnitCtx, p: WispPieces, commands: &[TurnCommand]) {
    for command in commands {
        turn(
            ctx,
            animation_piece(p, command.piece),
            command.axis,
            command.target,
            command.speed,
        );
    }
}

async fn wisp_create(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    let p = state.borrow().pieces;
    // Scripts/wisp.lua blocks in PlayAnimation("idle") before it starts
    // Float_Body. Lua's Sleep(20 * 33 ms) resumes at this engine boundary;
    ctx.sleep_frames(23).await;
    apply_wisp_idle_frame_20(&ctx, p);
    ctx.sleep_frames(10).await;
    ctx.spawn(TaskDefinition::with_state(
        "partsneeded-wisp-float-body",
        state,
        wisp_float_body,
    ));
}

#[allow(clippy::excessive_precision, clippy::needless_borrow)]
fn apply_wisp_idle_frame(ctx: &UnitCtx, p: WispPieces) {
    // This is wisp_idle.lua's time=0 keyframe.  Its Move values are relative
    // to each piece's authored local offset, matching MoveNow's engine
    // semantics; constructSkeleton is what made the original Lua animation
    // use these relative values at runtime.
    turn(&ctx, p.arm_r, Axis::X, 0.355667, 0.862845);
    turn(&ctx, p.arm_r, Axis::Y, 1.254724, 0.897136);
    turn(&ctx, p.arm_r, Axis::Z, 0.244023, 0.349930);
    turn(&ctx, p.crus_r, Axis::X, 0.0, 0.0);
    turn(&ctx, p.crus_r, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.crus_r, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.thigh_r, Axis::X, -0.199088, 0.298632);
    turn(&ctx, p.thigh_r, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.thigh_r, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.hair3, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hair3, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hair3, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.hair3, Axis::X, -8.604626, 0.0);
    move_piece(&ctx, p.hair3, Axis::Y, 8.172135, 0.0);
    move_piece(&ctx, p.hair3, Axis::Z, 0.000009, 0.0);
    turn(&ctx, p.foot_l, Axis::X, 0.0, 0.0);
    turn(&ctx, p.foot_l, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.foot_l, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.hand_l, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hand_l, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hand_l, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.foot_r, Axis::X, 0.0, 0.0);
    turn(&ctx, p.foot_r, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.foot_r, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.eyes, Axis::X, 0.0, 0.0);
    turn(&ctx, p.eyes, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.eyes, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.eyes, Axis::X, 0.000001, 0.0);
    move_piece(&ctx, p.eyes, Axis::Y, 0.000000, 0.0);
    move_piece(&ctx, p.eyes, Axis::Z, 2.617365, 0.0);
    turn(&ctx, p.mouth, Axis::X, 0.0, 0.0);
    turn(&ctx, p.mouth, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.mouth, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.mouth, Axis::X, -0.000001, 0.0);
    move_piece(&ctx, p.mouth, Axis::Y, 0.000001, 0.0);
    move_piece(&ctx, p.mouth, Axis::Z, 2.617341, 0.0);
    turn(&ctx, p.body, Axis::X, 0.0, 0.0);
    turn(&ctx, p.body, Axis::Y, -0.189330, 0.283996);
    move_piece(&ctx, p.body, Axis::X, 0.000000, 0.0);
    move_piece(&ctx, p.body, Axis::Y, 0.000000, 0.0);
    move_piece(&ctx, p.body, Axis::Z, 1.735964, 2.603951);
    turn(&ctx, p.hand_r, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hand_r, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hand_r, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.nose, Axis::X, 0.0, 0.0);
    turn(&ctx, p.nose, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.nose, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.nose, Axis::X, 0.000001, 0.0);
    move_piece(&ctx, p.nose, Axis::Y, 0.000000, 0.0);
    move_piece(&ctx, p.nose, Axis::Z, 2.617365, 0.0);
    turn(&ctx, p.ears, Axis::X, 0.0, 0.0);
    turn(&ctx, p.ears, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.ears, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.ears, Axis::X, 0.000001, 0.0);
    move_piece(&ctx, p.ears, Axis::Y, 0.000000, 0.0);
    move_piece(&ctx, p.ears, Axis::Z, 2.617365, 0.0);
    turn(&ctx, p.forearm_l, Axis::X, 0.0, 0.0);
    turn(&ctx, p.forearm_l, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.forearm_l, Axis::Z, -0.687082, 0.0);
    turn(&ctx, p.thigh_l, Axis::X, 0.0, 0.0);
    turn(&ctx, p.thigh_l, Axis::Y, -0.350946, 0.526419);
    turn(&ctx, p.thigh_l, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.crus_l, Axis::Y, 0.862288, 1.293431);
    turn(&ctx, p.crus_l, Axis::X, 0.0, 0.0);
    turn(&ctx, p.crus_l, Axis::Z, 0.0, 0.0);
    turn(&ctx, p.hair1, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hair1, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hair1, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.hair1, Axis::X, 8.436891, 0.0);
    move_piece(&ctx, p.hair1, Axis::Y, -8.869379, 0.0);
    move_piece(&ctx, p.hair1, Axis::Z, 0.000017, 0.0);
    turn(&ctx, p.forearm_r, Axis::X, 0.0, 0.0);
    turn(&ctx, p.forearm_r, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.forearm_r, Axis::Z, 1.053308, 0.0);
    turn(&ctx, p.hair2, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hair2, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hair2, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.hair2, Axis::X, 7.409627, 0.0);
    move_piece(&ctx, p.hair2, Axis::Y, 9.622904, 0.0);
    move_piece(&ctx, p.hair2, Axis::Z, 0.000043, 0.0);
    turn(&ctx, p.hair4, Axis::X, 0.0, 0.0);
    turn(&ctx, p.hair4, Axis::Y, 0.0, 0.0);
    turn(&ctx, p.hair4, Axis::Z, 0.0, 0.0);
    move_piece(&ctx, p.hair4, Axis::X, 0.179462, 0.0);
    move_piece(&ctx, p.hair4, Axis::Y, 2.392741, 0.0);
    move_piece(&ctx, p.hair4, Axis::Z, 0.000017, 0.0);
    turn(&ctx, p.arm_l, Axis::X, -0.318421, 0.802885);
    turn(&ctx, p.arm_l, Axis::Y, 0.940668, 1.028958);
    turn(&ctx, p.arm_l, Axis::Z, -0.115121, 0.361067);
}

#[allow(clippy::excessive_precision)]
fn apply_wisp_idle_frame_20(ctx: &UnitCtx, p: WispPieces) {
    // wisp_idle.lua's second keyframe. Move targets are relative to the
    // authored piece offsets, exactly as the time=0 keyframe above.
    turn(ctx, p.arm_r, Axis::X, 0.221635, 0.402096);
    turn(ctx, p.arm_r, Axis::Y, 1.688724, 1.301999);
    turn(ctx, p.arm_r, Axis::Z, 0.369659, 0.376906);
    turn(ctx, p.forearm_l, Axis::X, 0.0, 0.0);
    turn(ctx, p.forearm_l, Axis::Y, 0.0, 0.0);
    turn(ctx, p.forearm_l, Axis::Z, -1.869209, 3.546380);
    turn(ctx, p.thigh_l, Axis::X, 0.291373, 0.874118);
    turn(ctx, p.thigh_l, Axis::Y, -0.126040, 0.674719);
    turn(ctx, p.thigh_l, Axis::Z, -0.043370, 0.130111);
    turn(ctx, p.forearm_r, Axis::X, 0.0, 0.0);
    turn(ctx, p.forearm_r, Axis::Y, 0.0, 0.0);
    turn(ctx, p.forearm_r, Axis::Z, 2.004571, 2.853787);
    turn(ctx, p.crus_l, Axis::X, 0.0, 0.0);
    turn(ctx, p.crus_l, Axis::Y, 0.193483, 2.006414);
    turn(ctx, p.crus_l, Axis::Z, 0.0, 0.0);
    turn(ctx, p.arm_l, Axis::X, -0.206460, 0.335883);
    turn(ctx, p.arm_l, Axis::Y, 1.519118, 1.735349);
    turn(ctx, p.arm_l, Axis::Z, -0.434828, 0.959122);
    turn(ctx, p.body, Axis::X, 0.0, 0.0);
    turn(ctx, p.body, Axis::Y, -0.036970, 0.457081);
    // Lua's constructSkeleton subtracts Body's authored X offset
    // (-3.278554) from this animation target before issuing Move().  The
    // CUS API receives the relative target, so the value is 1.428180 here.
    move_piece(ctx, p.body, Axis::X, 1.428180, 4.284539);
    move_piece(ctx, p.body, Axis::Y, 0.082978, 0.248933);
    move_piece(ctx, p.body, Axis::Z, 0.009945, 5.178057);
}

async fn wisp_idle(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    apply_wisp_idle_frame(&ctx, p);
    ctx.sleep_frames(20).await;
    apply_wisp_idle_frame_20(&ctx, p);
    ctx.sleep_frames(10).await;
}

fn start_wisp_movement(ctx: &UnitCtx, state: Rc<RefCell<WispAnimationState>>) {
    let mask = state.borrow().mask;
    ctx.signal(SIG_WALK);
    if mask >= 0 && mask & 8 != 0 {
        ctx.signal(SIG_FLOAT);
        ctx.spawn(TaskDefinition::with_state(
            "partsneeded-wisp-walk-legs",
            Rc::clone(&state),
            wisp_walk_legs,
        ));
        ctx.spawn(TaskDefinition::with_state(
            "partsneeded-wisp-walk-arms",
            state,
            wisp_walk_arms,
        ));
    } else if mask >= 0 && mask & 4 != 0 {
        ctx.spawn(TaskDefinition::with_state(
            "partsneeded-wisp-float-arms",
            state,
            wisp_float_arms,
        ));
    }
}

#[allow(clippy::excessive_precision)]
async fn wisp_float_body(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_FLOAT);
    let p = state.borrow().pieces;
    // wisp_hover_body.lua, with Body's authored Z offset (24.52532) removed.
    const FRAMES: &[(u64, f32, f32)] = &[
        (0, 1.600370, 7.495241),
        (6, 3.688683, 12.529873),
        (11, 5.483999, 8.976583),
        (17, 5.233107, 1.075254),
        (24, 2.997091, 11.180077),
        (30, 1.468426, 9.171993),
        (35, 0.792952, 2.894889),
        (42, -0.648779, 8.650383),
        (47, -1.872648, 7.343216),
        (52, -2.478926, 3.031387),
        (58, -1.123247, 8.134071),
        (63, 0.686747, 7.757116),
    ];
    loop {
        for (index, &(time, target, speed)) in FRAMES.iter().enumerate() {
            move_piece(&ctx, p.body, Axis::Z, target, speed);
            let next = FRAMES.get(index + 1).map_or(70, |frame| frame.0);
            ctx.sleep_frames(next - time).await;
        }
    }
}

fn apply_wisp_walk_base_pose(ctx: &UnitCtx, p: WispPieces) {
    // These are the time=0 Move commands in wisp_walk_legs.lua after the Lua
    // constructSkeleton offset conversion. They are not optional: the walk
    // animation intentionally puts the legs and face on their authored
    // walking baseline before the first interpolated keyframe.
    move_piece(ctx, p.nose, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.nose, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.nose, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.hair1, Axis::X, 8.436891, 0.0);
    move_piece(ctx, p.hair1, Axis::Y, -8.869379, 0.0);
    move_piece(ctx, p.hair1, Axis::Z, 0.000017, 0.0);
    move_piece(ctx, p.thigh_r, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.thigh_r, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.thigh_r, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.body, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.body, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.body, Axis::Z, -2.050539, 8.359875);
    move_piece(ctx, p.thigh_l, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.thigh_l, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.thigh_l, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.head, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.head, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.head, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.eyes, Axis::X, 0.000001, 0.0);
    move_piece(ctx, p.eyes, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.eyes, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.foot_l, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.foot_l, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.foot_l, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.crus_r, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.crus_r, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.crus_r, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.crus_l, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.crus_l, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.crus_l, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.ears, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.ears, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.ears, Axis::Z, 2.617365, 0.0);
    move_piece(ctx, p.hair3, Axis::X, -8.604626, 0.0);
    move_piece(ctx, p.hair3, Axis::Y, 8.172135, 0.0);
    move_piece(ctx, p.hair3, Axis::Z, 0.000009, 0.0);
    move_piece(ctx, p.mouth, Axis::X, -0.000001, 0.0);
    move_piece(ctx, p.mouth, Axis::Y, 0.000001, 0.0);
    move_piece(ctx, p.mouth, Axis::Z, 2.617341, 0.0);
    move_piece(ctx, p.hair2, Axis::X, 7.409627, 0.0);
    move_piece(ctx, p.hair2, Axis::Y, 9.622904, 0.0);
    move_piece(ctx, p.hair2, Axis::Z, 0.000043, 0.0);
    move_piece(ctx, p.hair4, Axis::X, 0.179462, 0.0);
    move_piece(ctx, p.hair4, Axis::Y, 2.392741, 0.0);
    move_piece(ctx, p.hair4, Axis::Z, 0.000017, 0.0);
    move_piece(ctx, p.foot_r, Axis::X, 0.0, 0.0);
    move_piece(ctx, p.foot_r, Axis::Y, 0.0, 0.0);
    move_piece(ctx, p.foot_r, Axis::Z, 2.617365, 0.0);
}

#[allow(clippy::excessive_precision)]
async fn wisp_walk_legs(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    // Sparse commands are intentional. In the Lua animation, an omitted
    // channel keeps its current animation; filling it with zero commands
    // changes the gait and cancels the previous interpolation.
    const FRAMES: &[(u64, &[TurnCommand])] = &[
        (
            0,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::Nose,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Nose,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Nose,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair1,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair1,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair1,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: 0.553389,
                    speed: 0.941258,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: -0.767710,
                    speed: 0.796178,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Head,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Head,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Head,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Eyes,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Eyes,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Eyes,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::Y,
                    target: 0.323116,
                    speed: 1.938694,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 0.206269,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Y,
                    target: 0.598818,
                    speed: 3.661160,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Ears,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Ears,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Ears,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair3,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair3,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair3,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Mouth,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Mouth,
                    axis: Axis::Y,
                    target: -0.259905,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Mouth,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair2,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair2,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair2,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair4,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair4,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::Hair4,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::Y,
                    target: 0.221161,
                    speed: 2.349040,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
        (
            5,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: 0.315018,
                    speed: 1.430225,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: -0.461077,
                    speed: 1.839800,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 0.809301,
                    speed: 3.618187,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Y,
                    target: 0.165657,
                    speed: 2.598968,
                },
            ],
        ),
        (
            10,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: -0.058811,
                    speed: 2.413597,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 1.411390,
                    speed: 3.612534,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: -0.396740,
                    speed: 4.270547,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::Y,
                    target: 0.014238,
                    speed: 1.853267,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
        (
            15,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: 0.260293,
                    speed: 1.914624,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 1.261474,
                    speed: 0.899493,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: -0.750437,
                    speed: 2.122183,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
        (
            20,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: 0.263994,
                    speed: 0.022204,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 0.523410,
                    speed: 4.428388,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Y,
                    target: 0.721554,
                    speed: 3.335384,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: -0.770156,
                    speed: 0.118317,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
        (
            25,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: -0.467843,
                    speed: 4.391023,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Y,
                    target: 0.206269,
                    speed: 0.951420,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Y,
                    target: 1.351868,
                    speed: 3.781881,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: -0.113266,
                    speed: 3.941344,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootL,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.042714,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::Y,
                    target: -0.170345,
                    speed: 1.174520,
                },
                TurnCommand {
                    piece: WispAnimationPiece::FootR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
        (
            30,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Y,
                    target: -0.635014,
                    speed: 1.003023,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Y,
                    target: 1.209011,
                    speed: 0.857137,
                },
                TurnCommand {
                    piece: WispAnimationPiece::CrusL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Y,
                    target: 0.396513,
                    speed: 3.058669,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ThighR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
            ],
        ),
    ];
    loop {
        for (index, &(time, commands)) in FRAMES.iter().enumerate() {
            if index == 0 {
                apply_wisp_walk_base_pose(&ctx, p);
            }
            apply_turn_commands(&ctx, p, commands);
            if index == 1 {
                move_piece(&ctx, p.body, Axis::Z, -0.694033, 8.139038);
            } else if index == 2 {
                move_piece(&ctx, p.body, Axis::Z, 1.056810, 10.505058);
            } else if index == 3 {
                move_piece(&ctx, p.body, Axis::Z, -1.987444, 9.132763);
            } else if index == 5 {
                move_piece(&ctx, p.body, Axis::Z, 0.912222, 17.397995);
            } else if index == 6 {
                move_piece(&ctx, p.body, Axis::Z, -0.394337, 7.839352);
            }
            let next = FRAMES.get(index + 1).map_or(35, |next| next.0);
            ctx.sleep_frames(next - time).await;
        }
    }
}

#[allow(clippy::excessive_precision)]
async fn wisp_walk_arms(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    const FRAMES: &[(u64, &[TurnCommand])] = &[
        (
            0,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::HandL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::HandL,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::HandL,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.845291,
                    speed: 0.019220,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: -0.439749,
                    speed: 2.987958,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.517597,
                    speed: 0.118091,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: -0.020689,
                    speed: 0.047744,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.322941,
                    speed: 0.745248,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.665447,
                    speed: 0.750137,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: -0.224734,
                    speed: 0.518617,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: 0.358980,
                    speed: 0.828414,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 0.767355,
                    speed: 1.068285,
                },
                TurnCommand {
                    piece: WispAnimationPiece::HandR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::HandR,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::HandR,
                    axis: Axis::Z,
                    target: 0.0,
                    speed: 0.0,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.921551,
                    speed: 0.000001,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.740675,
                    speed: 3.724443,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: 0.005804,
                    speed: 0.0,
                },
            ],
        ),
        (
            5,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.984199,
                    speed: 0.520906,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: -0.643628,
                    speed: 0.764548,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.077189,
                    speed: 1.651531,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.905852,
                    speed: 0.058871,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.827111,
                    speed: 0.324135,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -0.033965,
                    speed: 0.149137,
                },
            ],
        ),
        (
            13,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.837310,
                    speed: 0.489631,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: -0.034834,
                    speed: 2.029315,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.376410,
                    speed: 0.997403,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: -0.064199,
                    speed: 0.535116,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: 0.103016,
                    speed: 0.853213,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 1.134575,
                    speed: 1.224067,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: -0.039136,
                    speed: 0.061490,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.206016,
                    speed: 0.389750,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.669665,
                    speed: 0.014061,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.919834,
                    speed: 0.046605,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.328922,
                    speed: 1.660630,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -0.110092,
                    speed: 0.253754,
                },
            ],
        ),
        (
            22,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.848494,
                    speed: 0.041941,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.058244,
                    speed: 0.349043,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.497916,
                    speed: 0.455647,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.240747,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.386309,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 1.230278,
                    speed: 0.358887,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.146760,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.772559,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.990506,
                    speed: 1.203154,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.921551,
                    speed: 0.006441,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.119935,
                    speed: 0.783704,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: 0.005804,
                    speed: 0.434610,
                },
            ],
        ),
    ];
    loop {
        for (index, &(time, commands)) in FRAMES.iter().enumerate() {
            apply_turn_commands(&ctx, p, commands);
            let next = FRAMES.get(index + 1).map_or(30, |next| next.0);
            ctx.sleep_frames(next - time).await;
        }
    }
}

#[allow(clippy::excessive_precision)]
async fn wisp_float_arms(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    // The complete sparse wisp_hover_arms.lua animation. An omitted command
    // must remain omitted: it lets the previous turn continue as in Lua.
    const FRAMES: &[(u64, &[TurnCommand])] = &[
        (
            0,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: 0.226893,
                    speed: 1.134464,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.680678,
                    speed: 0.959931,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -1.448623,
                    speed: 3.490659,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: -0.157080,
                    speed: 4.974189,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: -0.279253,
                    speed: 1.396263,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.174533,
                    speed: 6.108652,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: -0.230642,
                    speed: 1.153210,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.685336,
                    speed: 0.978023,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: 1.453831,
                    speed: 3.444321,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: 0.154882,
                    speed: 4.968613,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: -0.282235,
                    speed: 1.411176,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 0.188406,
                    speed: 6.053595,
                },
            ],
        ),
        (
            6,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.191986,
                    speed: 2.094395,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.488692,
                    speed: 0.959931,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -0.890118,
                    speed: 2.792527,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 0.025823,
                    speed: 0.914514,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.296706,
                    speed: 2.879793,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.139626,
                    speed: 0.174533,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.202616,
                    speed: 2.166291,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.488890,
                    speed: 0.982235,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: 0.894441,
                    speed: 2.796952,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: -0.089640,
                    speed: 0.611306,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: -0.316113,
                    speed: 0.084695,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 0.089345,
                    speed: 0.247651,
                },
            ],
        ),
        (
            12,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.052360,
                    speed: 0.698132,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.349066,
                    speed: 0.698132,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -0.104720,
                    speed: 3.926991,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 0.087266,
                    speed: 0.307216,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: -0.314159,
                    speed: 3.054326,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.087266,
                    speed: 0.261799,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.068022,
                    speed: 0.672970,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.352486,
                    speed: 0.682016,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: 0.121332,
                    speed: 3.865546,
                },
            ],
        ),
        (
            18,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: -0.279253,
                    speed: 1.134464,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.820305,
                    speed: 2.356195,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: 0.750492,
                    speed: 4.276057,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 0.383972,
                    speed: 1.483530,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: -0.244346,
                    speed: 0.349066,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.349066,
                    speed: 1.308997,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.280070,
                    speed: 1.060237,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.832402,
                    speed: 2.399576,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.761214,
                    speed: 4.412727,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: -1.028630,
                    speed: 1.482615,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: -0.139897,
                    speed: 0.278235,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 0.952302,
                    speed: 1.362563,
                },
            ],
        ),
        (
            24,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: 0.052360,
                    speed: 0.765260,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: 0.418879,
                    speed: 0.765260,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 1.012291,
                    speed: 1.449966,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.139626,
                    speed: 0.886090,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -0.942478,
                    speed: 1.369412,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: -0.056677,
                    speed: 0.777107,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.833298,
                    speed: 0.002070,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: -0.431629,
                    speed: 0.760581,
                },
            ],
        ),
        (
            37,
            &[
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.112200,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Y,
                    target: 0.872665,
                    speed: 0.112200,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmL,
                    axis: Axis::Z,
                    target: -0.750492,
                    speed: 2.505794,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::X,
                    target: 0.837758,
                    speed: 0.373999,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.299199,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmL,
                    axis: Axis::Z,
                    target: -1.396263,
                    speed: 0.972398,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::X,
                    target: 0.0,
                    speed: 0.121450,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Y,
                    target: 0.880941,
                    speed: 0.102092,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ArmR,
                    axis: Axis::Z,
                    target: 0.764967,
                    speed: 2.564134,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::X,
                    target: -0.838840,
                    speed: 0.406691,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Y,
                    target: 0.0,
                    speed: 0.299781,
                },
                TurnCommand {
                    piece: WispAnimationPiece::ForeArmR,
                    axis: Axis::Z,
                    target: 1.399125,
                    speed: 0.957478,
                },
            ],
        ),
    ];
    loop {
        for (index, &(time, commands)) in FRAMES.iter().enumerate() {
            apply_turn_commands(&ctx, p, commands);
            let next = FRAMES.get(index + 1).map_or(51, |next| next.0);
            ctx.sleep_frames(next - time).await;
        }
    }
}

fn unit_piece(unit: UnitId, name: &str) -> Piece {
    spring::get_unit_piece_map(unit.0)
        .ok()
        .and_then(|pieces| {
            pieces
                .into_iter()
                .find(|piece| piece.name == name)
                .map(|piece| Piece(piece.piece_num - 1))
        })
        .unwrap_or(Piece::INVALID)
}

async fn gate_open(ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_GATE);
    ctx.sleep(Duration::from_millis(500)).await;
    crate::sound::play_unit_sound("sounds/gate.ogg", 40.0, ctx.unit().0);
    ctx.move_piece(
        unit_piece(ctx.unit(), "Cube41_extract45_GATE"),
        Axis::Z,
        -168.0,
        180.0,
    );
}

async fn gate_close(ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_GATE);
    crate::sound::play_unit_sound("sounds/gate.ogg", 5.0, ctx.unit().0);
    let gate = unit_piece(ctx.unit(), "Cube41_extract45_GATE");
    ctx.move_piece(gate, Axis::Z, 0.0, 190.0);
    ctx.wait_for_move(gate, Axis::Z).await;
}

async fn lever_on(ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/lever.ogg", 0.6, ctx.unit().0);
    turn(
        &ctx,
        unit_piece(ctx.unit(), "Base_001"),
        Axis::X,
        -0.6,
        15.0,
    );
}

async fn lever_off(ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/lever.ogg", 0.6, ctx.unit().0);
    turn(&ctx, unit_piece(ctx.unit(), "Base_001"), Axis::X, 0.6, 15.0);
}

async fn plate_on(ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/plate.01.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(unit_piece(ctx.unit(), "Button"), Axis::Z, -2.0, 15.0);
}

async fn plate_off(ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/plate.01.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(unit_piece(ctx.unit(), "Button"), Axis::Z, 0.0, 15.0);
}

async fn cage_open(ctx: UnitCtx) {
    if spring::get_unit_rules_param_f32(ctx.unit().0, "do_kill") == Ok(1.0) {
        return;
    }
    crate::sound::play_unit_sound("sounds/cage.02.ogg", 1.0, ctx.unit().0);
    let bars = unit_piece(ctx.unit(), "BarsFront");
    ctx.move_piece(bars, Axis::Z, -200.0, 150.0);
    ctx.wait_for_move(bars, Axis::Z).await;
}

async fn cage_close(ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/cage.02.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(unit_piece(ctx.unit(), "BarsFront"), Axis::Z, 0.0, 150.0);
}

impl PartsUnitScript {
    fn set_piece_mask(&mut self, ctx: &UnitCtx, mask: i32) {
        let p = self.pieces;
        let old_mask = self.animation.borrow().mask;
        for piece in p.all {
            ctx.hide(piece);
        }
        self.animation.borrow_mut().mask = mask;
        if mask < 0 {
            return;
        }
        ctx.show(p.body);
        ctx.show(p.head);
        if mask & 1 != 0 {
            ctx.show(p.eyes);
        }
        if mask & 2 != 0 {
            ctx.show(p.ears);
        }
        if mask & 4 != 0 {
            for piece in p.arms {
                ctx.show(piece);
            }
        }
        if mask & 8 != 0 {
            for piece in p.legs {
                ctx.show(piece);
            }
        }
        if mask & 16 != 0 {
            ctx.show(p.nose);
        }
        if mask & 32 != 0 {
            ctx.show(p.mouth);
        }

        // SetPieceVisible() in the Lua script calls updateMovement() whenever
        // the arms or legs group changes: a moving wisp switches to the gait
        // its limbs allow (walk with legs, float_arms "swim" with arms only),
        // a standing one to idle.  Without this, a wisp that gains arms while
        // already moving keeps its old animation, and the first spirit ->
        // corporal switch leaves the body frozen at its spirit-form pose.
        let limbs_changed = old_mask < 0 || (old_mask ^ mask) & (4 | 8) != 0;
        if self.kind == Some(ScriptKind::Wisp) && limbs_changed && (mask & (4 | 8)) != 0 {
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
    static PENDING_MASKS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
}

/// Queue a wisp piece mask (wisp.lua's SetPieceVisible groups) for the next
/// Core-CUS tick.  The rules run inside this same module, so routing the call
/// through the engine (`call_unit_script`) is a nested call into a module that
/// is already running a gadget callin; the engine rejects it (ApiError 1) and
/// the mask never arrived, leaving the wisp's animation state in spirit form.
pub fn queue_piece_mask(unit: i32, mask: i32) {
    if unit >= 0 {
        PENDING_MASKS.with(|pending| pending.borrow_mut().push((unit, mask)));
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
        for (unit_id, mask) in pending {
            let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
                continue;
            };
            self.sync_instance_frame(handle);
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.set_piece_mask(&ctx, mask));
            });
        }
    }

    fn current_game_frame() -> u64 {
        spring::game::get_game_frame(0)
            .map(|value| (value.low16 | (value.high16 << 16)) as u64)
            .unwrap_or(0)
    }

    fn sync_instance_frame(&mut self, handle: CusHandle) {
        let frame = Self::current_game_frame();
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
            let mask = arguments.first().copied().unwrap_or(-1.0) as i32;
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.set_piece_mask(&ctx, mask));
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
