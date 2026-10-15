use partsneeded_shared::BodyParts;
use spring_addons::rules::Gadget;

use crate::state::Global;

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
            let direction = if frame == 105 { (1.0, 0.0) } else { (0.0, 0.0) };
            ctx.global().wisp.access_mut().steering.set(direction);
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
    crate::units::destroy(
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
