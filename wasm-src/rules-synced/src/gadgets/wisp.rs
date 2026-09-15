use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::wisp;

/// Frames a movement message stays valid.
///
/// The visual conformance run deliberately sends through the real LuaUI/WasmUI
/// boundary, but its software-rendered UI can pause for more than six synced
/// frames between updates. Keep that harness from introducing a synthetic
/// StopMoving/StartMoving transition; production retains the Lua-compatible
/// six-frame timeout.
#[cfg(feature = "unit-script-probe")]
const MOVE_STALE_FRAMES: i32 = 120;
#[cfg(not(feature = "unit-script-probe"))]
const MOVE_STALE_FRAMES: i32 = 6;

/// The player wisp controller, ported from `unit_wisp_controller.lua`.
#[derive(Default)]
pub struct WispGadget;

impl WispGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for WispGadget {
    fn name(&self) -> &'static str {
        "Wisp"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        let global = ctx.global();
        let frame = global.clock.access().frame;
        let mut wisp = global.wisp.access_mut();
        if wisp.id >= 0 {
            wisp::cancel_upward_velocity(&wisp);
            #[cfg(feature = "headless-integration")]
            {
                let move_goal_accepted = wisp::move_wisp(&wisp);
                if wisp.movement.is_some() && !move_goal_accepted {
                    let _ = spring::echo("[PartsNeeded-Rust-Test] move goal was rejected", "");
                }
            }
            #[cfg(not(feature = "headless-integration"))]
            {
                let _ = wisp::move_wisp(&wisp);
            }
            let collected = wisp::collect_body_parts(&mut wisp);
            global.pending.access_mut().destroys.extend(collected);
        }
        // The UI samples one message per frame but they arrive through the
        // network layer, so a 2 frame window dropped inputs whenever delivery
        // hiccuped and the wisp stuttered to a halt.
        if wisp.movement.is_some() && frame > wisp.movement_frame + MOVE_STALE_FRAMES {
            wisp.movement = None;
        }
        let mut pending = global.pending.access_mut();
        if wisp.id >= 0 && wisp.movement.is_none() && !wisp.stop_issued {
            wisp.stop_issued = true;
            pending.stop_wisp = Some(wisp.id);
        }
        wisp::rotate_body_parts(&wisp);

        if wisp.pieces_dirty {
            wisp.pieces_dirty = false;
            pending.wisp_pieces = Some((wisp.id, wisp.spirit, wisp.collected_parts.clone()));
        }
    }
}
