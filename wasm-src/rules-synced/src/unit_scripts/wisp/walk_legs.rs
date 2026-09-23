//! wisp_walk_legs.lua.

use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, UnitCtx};

use super::idle::apply_wisp_walk_base_pose;
use super::{
    SIG_WALK, TurnCommand, WispAnimationPiece, WispAnimationState, apply_turn_commands, move_piece,
};

#[allow(clippy::excessive_precision)]
pub(crate) async fn wisp_walk_legs(state: Rc<RefCell<WispAnimationState>>, ctx: UnitCtx) {
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
