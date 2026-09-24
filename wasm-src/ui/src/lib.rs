extern crate alloc;

pub mod audio;
pub mod commands;
pub mod defs;
pub mod shader;
pub mod state;
pub mod widgets;

/// Draw the player wisp's model (its body form), or not (a spirit, drawn as a
/// sprite instead).
pub fn show_body(wisp: i32, shown: bool) {
    use spring::ResultExt;
    if wisp >= 0 {
        spring::unsynced_ctrl::set_unit_no_draw(wisp, !shown).log_err_once("showing the body");
    }
}

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
