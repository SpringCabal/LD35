pub fn play_unit_sound(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    name: &str,
    volume: f32,
    unit_id: i32,
) {
    let position = spring
        .units_info()
        .get_unit_position(unit_id, Default::default())
        .unwrap_or_else(|_| spring::Float3::default());
    let _ = spring
        .sound()
        .play_sound_file(name, volume, position, spring::Float3::default(), 0);
}
