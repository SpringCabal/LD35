use partsneeded_shared::BodyParts;
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Synced control for the runtime unit-script comparison.
///
/// The UI arms this scenario with one message. All state changes after that
/// happen at fixed game frames, which removes render/update cadence from the
/// pose comparison while still exercising the normal unit and CUS callbacks.
#[derive(Default)]
pub struct UnitScriptProbeGadget;

impl UnitScriptProbeGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for UnitScriptProbeGadget {
    fn name(&self) -> &'static str {
        "UnitScriptProbe"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if !ctx.global().unit_script_probe.access().active {
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
            assemble(ctx.global(), frame);
        } else if frame == 46 {
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.spirit = true;
            wisp.pieces_dirty = true;
            crate::state::param("spiritMode", 1.0);
        } else if frame == 89 {
            let mut wisp = ctx.global().wisp.access_mut();
            wisp.spirit = false;
            wisp.form_frame = frame;
            wisp.pieces_dirty = true;
            crate::state::param("spiritMode", 0.0);
        } else if (105..=170).contains(&frame) {
            let mut wisp = ctx.global().wisp.access_mut();
            if wisp.id >= 0 {
                wisp.movement = Some((1.0, 0.0));
                wisp.movement_frame = frame;
                wisp.stop_issued = false;
            }
        }
    }
}

fn assemble(global: &Global, frame: i32) {
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
            if let Some(part) = crate::wisp::body_part(def.0) {
                parts.push((id, part));
            }
        }
    }

    let mut collected = BodyParts::NONE;
    for &(_, part) in &parts {
        collected.insert(part);
        crate::state::param(&alloc::format!("has_{}", part.name()), 1.0);
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
    global
        .pending
        .access_mut()
        .destroys
        .extend(parts.into_iter().map(|(id, _)| (id, true, wisp_id)));
    let _ = spring::echo(
        &alloc::format!("[UnitScriptProbeAssembly] COLLECTED frame={frame} parts=6"),
        "",
    );
}
