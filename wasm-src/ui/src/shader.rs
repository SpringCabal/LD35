pub fn create(
    spring: &spring::Spring<'_, spring::Ui>,
    fragment: &str,
    samplers: &[&str],
) -> Option<u32> {
    let id = spring
        .gfx()
        .create_shader("", "", "", "", "", fragment, "", Default::default())
        .ok()?
        .shader_id;
    if id == 0 {
        return None;
    }
    // Match the previous helper's shader binding/reset on successful compile.
    let _ = spring.gfx().use_shader(id);
    let _ = spring.gfx().use_shader(0);
    if !samplers.is_empty() {
        let _ = spring.gfx().use_shader(id);
        for (unit, name) in samplers.iter().enumerate() {
            if let Ok(location) = spring.gfx().get_uniform_location(id, name) {
                let _ = spring.gfx().uniform_int(location, &[unit as i32], 1);
            }
        }
        let _ = spring.gfx().use_shader(0);
    }
    Some(id)
}

pub fn location(spring: &spring::Spring<'_, spring::Ui>, shader: u32, name: &str) -> i32 {
    spring
        .gfx()
        .get_uniform_location(shader, name)
        .unwrap_or(-1)
}
