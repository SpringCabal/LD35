use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, Duration, InitCtx, Piece, UnitCtx};

use super::SIG_GATE;
use super::wisp::turn;

#[derive(Clone, Copy, Default)]
pub(crate) struct PropPieces {
    pub(crate) gate: Piece,
    pub(crate) lever: Piece,
    pub(crate) button: Piece,
    pub(crate) bars: Piece,
}

impl PropPieces {
    pub(crate) fn from_init(ctx: &InitCtx<'_>) -> Self {
        Self {
            gate: ctx.piece("Cube41_extract45_GATE"),
            lever: ctx.piece("Base_001"),
            button: ctx.piece("Button"),
            bars: ctx.piece("BarsFront"),
        }
    }
}

pub(crate) type Props = Rc<RefCell<PropPieces>>;

pub(crate) fn props(state: &Props) -> PropPieces {
    *state.borrow()
}

pub(crate) async fn gate_open(state: Props, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_GATE);
    ctx.sleep(Duration::from_millis(500)).await;
    crate::sound::play_unit_sound("sounds/gate.ogg", 40.0, ctx.unit().0);
    ctx.move_piece(props(&state).gate, Axis::Z, -168.0, 180.0);
}

pub(crate) async fn gate_close(state: Props, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_GATE);
    crate::sound::play_unit_sound("sounds/gate.ogg", 5.0, ctx.unit().0);
    let gate = props(&state).gate;
    ctx.move_piece(gate, Axis::Z, 0.0, 190.0);
    ctx.wait_for_move(gate, Axis::Z).await;
}

pub(crate) async fn lever_on(state: Props, ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/lever.ogg", 0.6, ctx.unit().0);
    turn(&ctx, props(&state).lever, Axis::X, -0.6, 15.0);
}

pub(crate) async fn lever_off(state: Props, ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/lever.ogg", 0.6, ctx.unit().0);
    turn(&ctx, props(&state).lever, Axis::X, 0.6, 15.0);
}

pub(crate) async fn plate_on(state: Props, ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/plate.01.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(props(&state).button, Axis::Z, -2.0, 15.0);
}

pub(crate) async fn plate_off(state: Props, ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/plate.01.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(props(&state).button, Axis::Z, 0.0, 15.0);
}

pub(crate) async fn cage_open(state: Props, ctx: UnitCtx) {
    if crate::cages::kills(ctx.unit().0) {
        return;
    }
    crate::sound::play_unit_sound("sounds/cage.02.ogg", 1.0, ctx.unit().0);
    let bars = props(&state).bars;
    ctx.move_piece(
        bars,
        Axis::Z,
        -crate::cages::BARS_TRAVEL,
        crate::cages::BARS_SPEED,
    );
    ctx.wait_for_move(bars, Axis::Z).await;
}

pub(crate) async fn cage_close(state: Props, ctx: UnitCtx) {
    crate::sound::play_unit_sound("sounds/cage.02.ogg", 1.0, ctx.unit().0);
    ctx.move_piece(props(&state).bars, Axis::Z, 0.0, crate::cages::BARS_SPEED);
}
