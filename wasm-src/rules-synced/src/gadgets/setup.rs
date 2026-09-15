use spring_addons::register_gadgets;
use spring_addons::rules::{Gadget, GadgetHandler};

use super::*;
use crate::state::{Global, param};

#[derive(Default)]
pub struct SetupGadget;

impl SetupGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for SetupGadget {
    fn name(&self) -> &'static str {
        "Setup"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        {
            let mut clock = ctx.global().clock.access_mut();
            clock.frame = frame;
            if clock.initialized {
                return;
            }
            clock.initialized = true;
        }
        {
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.id = -1;
            wisp.spirit = true;
        }
        param("spiritMode", 1.0);
        param("game_over", 0.0);
        param("alive_wisps", 0.0);
        param("saved_wisps", 0.0);
        param("killed_wisps", 0.0);
    }
}

pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(handler, [SetupGadget::new(), LevelGadget::new(),]);

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

    register_gadgets!(handler, [LifecycleGadget::new(), InputGadget::new()]);

    register_gadgets!(handler, [ReaperGadget::new()]);
}
