use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use super::*;
use crate::state::Global;

/// Gadgets run in registration order.  The reaper goes last: it performs the
/// engine callouts the others queued this frame, once none of them holds a
/// resource.
pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(handler, [LevelGadget::new()]);

    #[cfg(feature = "unit-script-probe")]
    register_gadgets!(handler, [UnitScriptProbeGadget::new()]);

    register_gadgets!(handler, [WispGadget::new(), TriggersGadget::new()]);

    // The unit-script comparison keeps simulating after all six parts are
    // collected, so it runs without the ending, as the Lua fixture replaces
    // game_end.lua for the same run.
    #[cfg(not(feature = "unit-script-probe"))]
    register_gadgets!(handler, [EndgameGadget::new()]);

    register_gadgets!(handler, [StatusGadget::new()]);

    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [IntegrationGadget::new()]);

    register_gadgets!(handler, [LifecycleGadget::new(), InputGadget::new()]);

    register_gadgets!(handler, [ReaperGadget::new()]);
}
