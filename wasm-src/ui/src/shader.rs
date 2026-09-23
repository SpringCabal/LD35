//! Fragment-shader boilerplate shared by the screen effects.

/// Build a fragment-only shader and bind each named sampler to the texture
/// unit of its index.  Returns the shader id, or `None` if it did not build.
pub fn create(fragment: &str, samplers: &[&str]) -> Option<u32> {
    let id = spring::gfx::create_shader(spring::gfx::ShaderCreateParams {
        fragment,
        ..Default::default()
    })
    .ok()?
    .shader_id;
    if !samplers.is_empty() {
        let _ = spring::gfx::use_shader(id);
        for (unit, name) in samplers.iter().enumerate() {
            if let Ok(location) = spring::gfx::get_uniform_location(id, name) {
                let _ = spring::gfx::uniform_int(location, &[unit as i32], 1);
            }
        }
        let _ = spring::gfx::use_shader(0);
    }
    Some(id)
}

/// A uniform's location, or -1 (which GL ignores) when it is absent.
pub fn location(shader: u32, name: &str) -> i32 {
    spring::gfx::get_uniform_location(shader, name).unwrap_or(-1)
}
