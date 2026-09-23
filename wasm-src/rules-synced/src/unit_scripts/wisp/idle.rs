//! wisp_idle.lua's keyframes and the walk's base pose.

use spring::cus::{Axis, UnitCtx};

use super::{WispPieces, move_piece, turn};

#[allow(clippy::excessive_precision, clippy::needless_borrow)]
pub(crate) fn apply_wisp_idle_frame(ctx: &UnitCtx, p: WispPieces) {
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
pub(crate) fn apply_wisp_idle_frame_20(ctx: &UnitCtx, p: WispPieces) {
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

pub(crate) fn apply_wisp_walk_base_pose(ctx: &UnitCtx, p: WispPieces) {
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
