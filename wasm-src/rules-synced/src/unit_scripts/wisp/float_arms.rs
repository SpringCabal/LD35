//! wisp_float_arms.lua: swimming with arms but no legs.

use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, UnitCtx};

use super::{SIG_WALK, TurnCommand, WispAnimationPiece, WispAnimationState, apply_turn_commands};

pub(crate) async fn wisp_float_arms(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_WALK);
    let p = state.borrow().pieces;
    loop {
        for (index, &(time, commands)) in FRAMES.iter().enumerate() {
            apply_turn_commands(&ctx, p, commands);
            let next = FRAMES.get(index + 1).map_or(51, |next| next.0);
            ctx.sleep_frames(next - time).await;
        }
    }
}

// The complete sparse wisp_hover_arms.lua animation. An omitted command
// must remain omitted: it lets the previous turn continue as in Lua.
#[allow(clippy::excessive_precision)]
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
