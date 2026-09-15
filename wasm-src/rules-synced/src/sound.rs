//! Sound helpers.
//!
//! Sound playback is unsynced-only, so this synced module cannot do it: the
//! form-change sound is played by the UI module instead, off the spirit flip it
//! already receives in api_sync.  What is left here is for positional sounds
//! that do work from synced code.

pub fn play_unit_sound(name: &str, volume: f32, unit_id: i32) {
    let position = spring::get_unit_position(unit_id, false, false)
        .unwrap_or_else(|_| spring::types::Float3::default());
    let _ =
        spring::sound::play_sound_file(name, volume, position, spring::types::Float3::default(), 0);
}
