use partsneeded_shared::BodyParts;
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Synced control for the runtime unit-script comparison.
///
/// The UI arms this scenario with one message. All state changes after that
/// happen at fixed game frames, which removes render/update cadence from the
/// pose comparison while still exercising the normal unit and CUS callbacks.
#[derive(Default)]
pub struct UnitScriptProbeGadget {
    active: core::cell::Cell<bool>,
}

impl UnitScriptProbeGadget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Gadget<Global> for UnitScriptProbeGadget {
    fn name(&self) -> &'static str {
        "UnitScriptProbe"
    }

    fn handle_lua_msg(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if data == b"probe_begin" {
            self.active.set(true);
        }
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if !self.active.get() {
            return;
        }

        if frame == 43 {
            // The Lua fixture picks up the six parts while entering corporal
            // form, then returns to spirit form at frame 46. Reproduce that
            // transition here before the measured switch at frame 89 so both
            // runtimes arrive with the same active unit-script tasks.
            {
                let mut wisp = ctx.global().wisp.access_mut();
                wisp.spirit = false;
                wisp.pieces_dirty = true;
            }
            assemble(ctx, frame);
        } else if frame == 46 {
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.spirit = true;
            wisp.pieces_dirty = true;
        } else if frame == 89 {
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.spirit = false;
            wisp.form_frame = frame;
            wisp.pieces_dirty = true;
        } else if frame == 105 || frame == 171 {
            // Walk right through the last sampled frame, 170, as the Lua
            // fixture's move goals do.
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.movement = (frame == 105).then_some((1.0, 0.0));
            wisp.stop_issued = false;
        }
    }
}

fn assemble(ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
    let global = ctx.global();
    let wisp_id = global.wisp.access().id;
    if wisp_id < 0 {
        return;
    }

    let mut parts = alloc::vec::Vec::new();
    if let Ok(units) = spring::get_all_units() {
        for id in units {
            let Ok(def) = spring::get_unit_def_id(id) else {
                continue;
            };
            if let Some(part) = crate::defs::defs().body_part(def.0) {
                parts.push((id, part));
            }
        }
    }

    let mut collected = BodyParts::NONE;
    for &(_, part) in &parts {
        collected.insert(part);
    }
    if !collected.is_complete() {
        let _ = spring::echo(
            &alloc::format!(
                "[UnitScriptProbeAssembly] ERROR frame={frame} expected=6 found={}",
                collected.len()
            ),
            "",
        );
        return;
    }

    {
        let mut wisp = global.wisp.access_mut();
        wisp.collected_parts = collected;
        wisp.body_parts.clear();
        wisp.pieces_dirty = true;
    }
    crate::units::destroy_later(
        ctx,
        parts
            .into_iter()
            .map(|(id, _)| (id, true, wisp_id))
            .collect(),
    );
    let _ = spring::echo(
        &alloc::format!("[UnitScriptProbeAssembly] COLLECTED frame={frame} parts=6"),
        "",
    );
}
