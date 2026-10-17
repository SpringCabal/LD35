use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(
    handler: &mut GadgetHandler<Global>,
    _spring: &spring::Spring<'_, spring::RulesSynced>,
) {
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
