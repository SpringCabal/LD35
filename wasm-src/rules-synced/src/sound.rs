pub fn play_unit_sound(name: &str, volume: f32, unit_id: i32) {
    let position = spring::get_unit_position(unit_id, false, false)
        .unwrap_or_else(|_| spring::types::Float3::default());
    let _ =
        spring::sound::play_sound_file(name, volume, position, spring::types::Float3::default(), 0);
}
