//! wisp_walk_arms.lua.

use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, UnitCtx};

use super::{SIG_WALK, TurnCommand, WispAnimationPiece, WispAnimationState, apply_turn_commands};

#[allow(clippy::excessive_precision)]
pub(crate) async fn wisp_walk_arms(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
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
