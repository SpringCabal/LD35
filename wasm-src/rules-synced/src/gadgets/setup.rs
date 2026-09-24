use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use super::*;
use crate::state::Global;

/// Gadgets run in registration order.  Callouts that raise callins
/// synchronously (creating, destroying and ordering units) are delayed with
/// `ctx.delay` until the frame's callins have returned.
pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(handler, [UnitScriptsGadget::new(), LevelGadget::new()]);

    #[cfg(feature = "unit-script-probe")]
    register_gadgets!(handler, [UnitScriptProbeGadget::new()]);

    register_gadgets!(
        handler,
        [
            WispGadget::new(),
            TriggersGadget::new(),
            EndgameGadget::new(),
            StatusGadget::new(),
        ]
    );

    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [IntegrationGadget::new()]);

    register_gadgets!(
        handler,
        [
            RosterGadget::new(),
            CollisionsGadget::new(),
            InputGadget::new()
        ]
    );
}
