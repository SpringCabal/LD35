use alloc::rc::Rc;
use core::cell::RefCell;

use partsneeded_shared::{BodyPart, BodyParts};
use spring::cus::{Angle, Axis, InitCtx, Piece, TaskDefinition, UnitCtx};

use super::{SIG_FLOAT, SIG_WALK};
use crate::gadgets::wisp::behavior::WISP_PIECE_GROUPS;

mod float_arms;
mod idle;
mod walk_arms;
mod walk_legs;

use float_arms::wisp_float_arms;
pub(crate) use idle::apply_wisp_idle_frame;
use idle::apply_wisp_idle_frame_20;
use walk_arms::wisp_walk_arms;
use walk_legs::wisp_walk_legs;

#[derive(Clone, Copy, Default)]
pub(crate) struct WispPieces {
    pub(crate) all: [Piece; 22],
    pub(crate) groups: [(Piece, Option<BodyPart>); WISP_PIECE_GROUPS.len()],
    pub(crate) body: Piece,
    pub(crate) head: Piece,
    pub(crate) eyes: Piece,
    pub(crate) ears: Piece,
    pub(crate) nose: Piece,
    pub(crate) mouth: Piece,
    pub(crate) thigh_l: Piece,
    pub(crate) thigh_r: Piece,
    pub(crate) crus_l: Piece,
    pub(crate) crus_r: Piece,
    pub(crate) foot_l: Piece,
    pub(crate) foot_r: Piece,
    pub(crate) arm_l: Piece,
    pub(crate) arm_r: Piece,
    pub(crate) forearm_l: Piece,
    pub(crate) forearm_r: Piece,
    pub(crate) hair1: Piece,
    pub(crate) hair2: Piece,
    pub(crate) hair3: Piece,
    pub(crate) hair4: Piece,
    pub(crate) hand_l: Piece,
    pub(crate) hand_r: Piece,
}

impl WispPieces {
    pub(crate) fn from_init(ctx: &InitCtx<'_>) -> Self {
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
            groups: WISP_PIECE_GROUPS.map(|(name, group)| (p(name), group)),
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

pub(crate) struct WispAnimationState {
    pub(crate) pieces: WispPieces,
    pub(crate) parts: Option<BodyParts>,
    pub(crate) moving: bool,
}

pub(crate) fn turn(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
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

pub(crate) fn move_piece(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if speed == 0.0 {
        ctx.move_now(piece, axis, target);
    } else {
        ctx.move_piece(piece, axis, target, speed);
    }
}

#[derive(Clone, Copy)]
pub(crate) enum WispAnimationPiece {
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
pub(crate) struct TurnCommand {
    piece: WispAnimationPiece,
    axis: Axis,
    target: f32,
    speed: f32,
}

pub(crate) fn animation_piece(p: WispPieces, piece: WispAnimationPiece) -> Piece {
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

pub(crate) fn apply_turn_commands(ctx: &UnitCtx, p: WispPieces, commands: &[TurnCommand]) {
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

pub(crate) async fn wisp_create(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    let p = state.borrow().pieces;
    ctx.sleep_frames(20).await;
    apply_wisp_idle_frame_20(&ctx, p);
    ctx.sleep_frames(10).await;
    ctx.spawn(TaskDefinition::with_state(
        "partsneeded-wisp-float-body",
        state,
        wisp_float_body,
    ));
}

pub(crate) async fn wisp_idle(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    apply_wisp_idle_frame(&ctx, p);
    ctx.sleep_frames(20).await;
    apply_wisp_idle_frame_20(&ctx, p);
    ctx.sleep_frames(10).await;
}

pub(crate) fn start_wisp_movement(ctx: &UnitCtx, state: Rc<RefCell<WispAnimationState>>) {
    let parts = state.borrow().parts.unwrap_or_default();
    ctx.signal(SIG_WALK);
    if parts.contains(BodyPart::Legs) {
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
    } else if parts.contains(BodyPart::Arms) {
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
