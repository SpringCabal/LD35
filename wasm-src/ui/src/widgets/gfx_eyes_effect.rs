use alloc::string::String;
use alloc::vec::Vec;
use spring::gl;
use spring_addons::ui::Widget;

use crate::shader;
use crate::state::{GAME_END_DURATION, Global, current_frame};

// The full gfx_eyes_effect.lua pipeline: bright pass, separable Gaussian blur,
// then the combine that grades, vignettes and grains the scene.  The wisp's
// halo is the bloom picking the sprite up as a glow source, not anything the
// sprite shader draws.
const EYES_EFFECT_FRAGMENT: &str = include_str!("../shaders/eyes_effect.frag");
const BLOOM_BRIGHT_FRAGMENT: &str = include_str!("../shaders/bloom_bright.frag");
const BLOOM_BLUR_H_FRAGMENT: &str = include_str!("../shaders/bloom_blur_h.frag");
const BLOOM_BLUR_V_FRAGMENT: &str = include_str!("../shaders/bloom_blur_v.frag");
const WISP_SPRITE_FRAGMENT: &str = include_str!("../shaders/wisp_sprite.frag");

/// gfx_eyes_effect.lua's option defaults.  The plain locals above them in that
/// file are dead: OnchangeFunc() overwrites all three from the options table
/// before the widget ever draws.
const MAX_BRIGHTNESS: f32 = 0.22;
const ILLUM_THRESHOLD: f32 = 0.33;
const BLUR_PASSES: i32 = 2;

/// The wisp billboard is a 40x40 elmo quad in unit_wisp_draw.lua.
const WISP_SPRITE_SIZE: f32 = 40.0;
const TRAIL_PARTICLE_LIFETIME: f32 = 24.0;
const MAX_TRAIL_PARTICLES: usize = 64;

/// The bloom chain's GL handles.
///
/// These live outside the widget because `render_to_texture` takes a bare `fn`
/// callback with no context argument, so each pass has to reach its own state.
#[derive(Default)]
struct Bloom {
    width: i32,
    height: i32,
    screen_tex: String,
    bright_tex_1: String,
    bright_tex_2: String,
    bright_shader: u32,
    blur_h_shader: u32,
    blur_v_shader: u32,
    combine_shader: u32,
    loc_bright_illum: i32,
    loc_bright_inv_rx: i32,
    loc_bright_inv_ry: i32,
    loc_blur_h_inv_rx: i32,
    loc_blur_h_radius: i32,
    loc_blur_v_inv_ry: i32,
    loc_blur_v_radius: i32,
    loc_combine_illum: i32,
    loc_combine_max: i32,
    loc_combine_time: i32,
    loc_combine_spirit: i32,
    loc_combine_noise: i32,
    loc_combine_game_over: i32,
    loc_combine_debug: i32,
    initialized: bool,
}

#[derive(Clone, Copy)]
struct TrailParticle {
    position: spring::Float3,
    velocity: spring::Float3,
    life: f32,
    seed: f32,
}

/// `gfx.shape` serializes fixed-width vertex attributes regardless of the
/// `has_*` flags. Keep those buffers populated even for the untextured,
/// unlit particle fan; empty normal/UV buffers trap inside the generated ABI.
fn particle_vertex(point: spring::Float3, color: [f32; 4]) -> spring::gfx::GfxVertexData {
    spring::gfx::GfxVertexData {
        vertex: vec![point.x, point.y, point.z],
        normal: vec![0.0, 1.0, 0.0],
        tex_coord: vec![0.0, 0.0],
        color: vec![color[0], color[1], color[2], color[3]],
        has_vertex: true,
        has_normal: false,
        has_tex_coord: false,
        has_color: true,
    }
}

thread_local! {
    static BLOOM: core::cell::RefCell<Bloom> = core::cell::RefCell::new(Bloom::default());
}

/// `gl.RenderToTexture(tex, fn)` equivalent of the Lua helper: bind the source
/// texture, draw a full-target quad, and let the bound shader do the work.
fn blit_pass() {
    let (width, height) = BLOOM.with_borrow(|bloom| (bloom.width, bloom.height));
    if width <= 1 || height <= 1 {
        return;
    }
    let _ = spring::gfx::matrix_mode(gl::TEXTURE);
    let _ = spring::gfx::load_identity(0);
    let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
    let _ = spring::gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
}

/// GL_RGB16F_ARB, the float format gfx_eyes_effect.lua asks for so the bright
/// pass can hold values above 1.0 without clipping.
const GL_RGB16F_ARB: u32 = 0x881B;

fn ensure_bloom(width: i32, height: i32) {
    BLOOM.with_borrow_mut(|bloom| {
        if bloom.width != width || bloom.height != height {
            for texture in [&bloom.screen_tex] {
                if !texture.is_empty() {
                    let _ = spring::gfx::delete_texture(texture);
                }
            }
            for texture in [&bloom.bright_tex_1, &bloom.bright_tex_2] {
                if !texture.is_empty() {
                    let _ = spring::gfx::delete_texture_fbo(texture);
                }
            }
            bloom.screen_tex.clear();
            bloom.bright_tex_1.clear();
            bloom.bright_tex_2.clear();
            bloom.width = width;
            bloom.height = height;

            if let Ok(created) = spring::gfx::create_texture(
                width,
                height,
                0,
                spring::gfx::TextureCreateParams {
                    target: gl::TEXTURE_2D,
                    format: 0,
                    border: 0,
                    min_filter: gl::LINEAR,
                    mag_filter: gl::LINEAR,
                    wrap_s: gl::CLAMP,
                    wrap_t: gl::CLAMP,
                    ..Default::default()
                },
            ) {
                bloom.screen_tex = created;
            }
            // Quarter resolution, as authored: the blur is wide enough that the
            // lost detail never shows, and it is what makes the pass cheap.
            for slot in 0..2 {
                if let Ok(created) = spring::gfx::create_texture(
                    width / 4,
                    height / 4,
                    0,
                    spring::gfx::TextureCreateParams {
                        target: gl::TEXTURE_2D,
                        format: GL_RGB16F_ARB,
                        border: 0,
                        min_filter: gl::LINEAR,
                        mag_filter: gl::LINEAR,
                        wrap_s: gl::CLAMP,
                        wrap_t: gl::CLAMP,
                        fbo: true,
                        ..Default::default()
                    },
                ) {
                    if slot == 0 {
                        bloom.bright_tex_1 = created;
                    } else {
                        bloom.bright_tex_2 = created;
                    }
                }
            }
        }

        if bloom.initialized {
            return;
        }
        bloom.initialized = true;

        if let Some(id) = shader::create(BLOOM_BRIGHT_FRAGMENT, &["texture0"]) {
            bloom.bright_shader = id;
            bloom.loc_bright_illum = shader::location(id, "illuminationThreshold");
            bloom.loc_bright_inv_rx = shader::location(id, "inverseRX");
            bloom.loc_bright_inv_ry = shader::location(id, "inverseRY");
        }
        if let Some(id) = shader::create(BLOOM_BLUR_H_FRAGMENT, &["texture0"]) {
            bloom.blur_h_shader = id;
            bloom.loc_blur_h_inv_rx = shader::location(id, "inverseRX");
            bloom.loc_blur_h_radius = shader::location(id, "fragKernelRadius");
        }
        if let Some(id) = shader::create(BLOOM_BLUR_V_FRAGMENT, &["texture0"]) {
            bloom.blur_v_shader = id;
            bloom.loc_blur_v_inv_ry = shader::location(id, "inverseRY");
            bloom.loc_blur_v_radius = shader::location(id, "fragKernelRadius");
        }
        if let Some(id) = shader::create(EYES_EFFECT_FRAGMENT, &["texture0", "texture1"]) {
            bloom.combine_shader = id;
            bloom.loc_combine_illum = shader::location(id, "illuminationThreshold");
            bloom.loc_combine_max = shader::location(id, "fragMaxBrightness");
            bloom.loc_combine_time = shader::location(id, "time");
            bloom.loc_combine_spirit = shader::location(id, "spiritAmount");
            bloom.loc_combine_noise = shader::location(id, "noiseAmount");
            bloom.loc_combine_game_over = shader::location(id, "gameOverAmount");
            bloom.loc_combine_debug = shader::location(id, "debugDraw");
        }
    });
}

/// The authored screen treatment and spirit sprites, ported from
/// `gfx_eyes_effect.lua` and `unit_wisp_draw.lua`.
#[derive(Default)]
pub struct GfxEyesEffectWidget {
    camera_projection: core::cell::RefCell<[f32; 16]>,
    camera_modelview: core::cell::RefCell<[f32; 16]>,
    trail_particles: core::cell::RefCell<Vec<TrailParticle>>,
    trail_last_position: core::cell::Cell<Option<spring::Float3>>,
    sprite_shader: core::cell::Cell<u32>,
    sprite_time: core::cell::Cell<i32>,
    sprite_spirit: core::cell::Cell<i32>,
    sprite_unit: core::cell::Cell<i32>,
    sprite_death: core::cell::Cell<i32>,
    game_end_start: core::cell::Cell<i32>,
}

impl GfxEyesEffectWidget {
    pub fn new() -> Self {
        Self {
            trail_last_position: core::cell::Cell::new(None),
            sprite_time: core::cell::Cell::new(-1),
            sprite_spirit: core::cell::Cell::new(-1),
            sprite_unit: core::cell::Cell::new(-1),
            sprite_death: core::cell::Cell::new(-1),
            game_end_start: core::cell::Cell::new(-1),
            ..Default::default()
        }
    }

    fn draw_eyes_effect(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        width: i32,
        height: i32,
    ) {
        if width <= 1 || height <= 1 {
            return;
        }
        ensure_bloom(width, height);
        let (screen_tex, bright_1, bright_2, bright, blur_h, blur_v, combine) =
            BLOOM.with_borrow(|bloom| {
                (
                    bloom.screen_tex.clone(),
                    bloom.bright_tex_1.clone(),
                    bloom.bright_tex_2.clone(),
                    bloom.bright_shader,
                    bloom.blur_h_shader,
                    bloom.blur_v_shader,
                    bloom.combine_shader,
                )
            });
        if screen_tex.is_empty() || combine == 0 {
            return;
        }
        if spring::gfx::copy_to_texture(&screen_tex, 0, 0, 0, 0, width, height, gl::TEXTURE_2D, 0)
            .is_err()
        {
            return;
        }

        let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
            enable: false,
            set_func: false,
            func: 0,
        });
        let _ = spring::gfx::depth_mask(false);
        let _ = spring::gfx::blending(true);
        let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
        let _ = spring::gfx::matrix_mode(gl::PROJECTION);
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::load_identity(0);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::load_identity(0);
        let _ = spring::gfx::matrix_mode(gl::TEXTURE);
        let _ = spring::gfx::load_identity(0);

        // The bloom chain only runs when every stage built; a missing FBO would
        // otherwise leave texture1 sampling whatever happened to be bound.
        let bloomed = !bright_1.is_empty()
            && !bright_2.is_empty()
            && bright != 0
            && blur_h != 0
            && blur_v != 0;
        if bloomed {
            let inverse_rx = 1.0 / width as f32;
            let inverse_ry = 1.0 / height as f32;
            // kernelRadius comes from the *view height* in the Lua widget: its
            // `vsx / 36.0` initialiser is overwritten by ViewResize with
            // vsy / 36.0 before the first draw.
            let kernel_radius = height as f32 / 36.0;

            let _ = spring::gfx::use_shader(bright);
            BLOOM.with_borrow(|bloom| {
                let _ = spring::gfx::uniform(bloom.loc_bright_inv_rx, &[inverse_rx], 1);
                let _ = spring::gfx::uniform(bloom.loc_bright_inv_ry, &[inverse_ry], 1);
                let _ = spring::gfx::uniform(bloom.loc_bright_illum, &[ILLUM_THRESHOLD], 1);
            });
            let _ = spring::gfx::bind_texture(&screen_tex, 0, true);
            let _ = spring::gfx::render_to_texture(&bright_1, blit_pass);

            for _ in 0..BLUR_PASSES {
                let _ = spring::gfx::use_shader(blur_h);
                BLOOM.with_borrow(|bloom| {
                    let _ = spring::gfx::uniform(bloom.loc_blur_h_inv_rx, &[inverse_rx], 1);
                    let _ = spring::gfx::uniform(bloom.loc_blur_h_radius, &[kernel_radius], 1);
                });
                let _ = spring::gfx::bind_texture(&bright_1, 0, true);
                let _ = spring::gfx::render_to_texture(&bright_2, blit_pass);

                let _ = spring::gfx::use_shader(blur_v);
                BLOOM.with_borrow(|bloom| {
                    let _ = spring::gfx::uniform(bloom.loc_blur_v_inv_ry, &[inverse_ry], 1);
                    let _ = spring::gfx::uniform(bloom.loc_blur_v_radius, &[kernel_radius], 1);
                });
                let _ = spring::gfx::bind_texture(&bright_2, 0, true);
                let _ = spring::gfx::render_to_texture(&bright_1, blit_pass);
            }
        }

        let (mut spirit, mut noise) = ctx.global().player.access().form.screen_amounts();
        let game_over = {
            let sequence = ctx.global().player.access().game_over_sequence;
            if !sequence {
                self.game_end_start.set(-1);
                0.0
            } else {
                let frame = crate::state::current_frame();
                let start = match self.game_end_start.get() {
                    start if start >= 0 => start,
                    _ => {
                        self.game_end_start.set(frame);
                        frame
                    }
                };
                let elapsed = (frame - start).max(0);
                let mut amount = (elapsed as f32 / GAME_END_DURATION as f32).min(1.0);
                if elapsed > GAME_END_DURATION * 2 {
                    amount =
                        (2.0 - elapsed as f32 / GAME_END_DURATION as f32 / 2.0).clamp(0.0, 1.0);
                    spirit = 2.0;
                    noise = 0.0;
                }
                amount
            }
        };
        let time = current_frame() as f32 / 30.0;

        let _ = spring::gfx::use_shader(combine);
        BLOOM.with_borrow(|bloom| {
            let _ = spring::gfx::uniform_int(bloom.loc_combine_debug, &[0], 1);
            let _ = spring::gfx::uniform(bloom.loc_combine_illum, &[ILLUM_THRESHOLD], 1);
            // With no bloom texture, zero the contribution rather than scale an
            // undefined sampler.
            let _ = spring::gfx::uniform(
                bloom.loc_combine_max,
                &[if bloomed { MAX_BRIGHTNESS } else { 0.0 }],
                1,
            );
            let _ = spring::gfx::uniform(bloom.loc_combine_time, &[time], 1);
            let _ = spring::gfx::uniform(bloom.loc_combine_spirit, &[spirit], 1);
            let _ = spring::gfx::uniform(bloom.loc_combine_noise, &[noise], 1);
            let _ = spring::gfx::uniform(bloom.loc_combine_game_over, &[game_over], 1);
        });
        let _ = spring::gfx::bind_texture(&screen_tex, 0, true);
        if bloomed {
            let _ = spring::gfx::bind_texture(&bright_1, 1, true);
        }
        let _ = spring::gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
        if bloomed {
            let _ = spring::gfx::bind_texture(&bright_1, 1, false);
        }
        let _ = spring::gfx::bind_texture(&screen_tex, 0, false);
        let _ = spring::gfx::use_shader(0);
        let _ = spring::gfx::matrix_mode(gl::PROJECTION);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::pop_attrib(0);
    }

    /// The wisp billboard, ported from `DrawWisp` in unit_wisp_draw.lua.
    ///
    /// The original draws a 40x40 elmo quad in world space with gl.Billboard.
    fn draw_wisp_sprite(&self, unit_id: i32, opacity: f32) {
        // unit_wisp_draw.lua calls Spring.GetUnitViewPosition(unitID) with no
        // second argument, i.e. the unit's base position.  Asking for midPos
        // instead lifted the whole billboard several pixels up the model.
        let Ok(position) = spring::unit_rendering::get_unit_view_position(unit_id, false) else {
            return;
        };
        self.ensure_sprite_shader();
        let shader = self.sprite_shader.get();
        if shader == 0 {
            return;
        }
        if self.sprite_time.get() >= 0 {
            let frame = current_frame() as f32 / 30.0;
            // unit_wisp_draw.lua quantises its time to a tenth of a second, so
            // the per-pixel noise resamples in visible steps instead of
            // shimmering every frame.
            let _ =
                spring::gfx::uniform(self.sprite_time.get(), &[(frame * 10.0).floor() / 10.0], 1);
        }
        if self.sprite_spirit.get() >= 0 {
            let _ = spring::gfx::uniform(self.sprite_spirit.get(), &[opacity], 1);
        }
        if self.sprite_unit.get() >= 0 {
            let _ = spring::gfx::uniform(self.sprite_unit.get(), &[unit_id as f32], 1);
        }
        if self.sprite_death.get() >= 0 {
            // unit_wisp_draw.lua fades every saved arrival through the public
            // is_dying/killed_frame/destroy_frame rules params. The player is
            // never in that lifecycle; absent params therefore naturally give
            // it the authored zero death amount.
            let death_amount = if spring::get_unit_rules_param_f32(unit_id, "is_dying")
                .ok()
                .is_some_and(|value| value == 1.0)
            {
                let destroy = spring::get_unit_rules_param_f32(unit_id, "destroy_frame").ok();
                let killed = spring::get_unit_rules_param_f32(unit_id, "killed_frame").ok();
                match (destroy, killed) {
                    (Some(destroy), Some(killed)) if destroy > killed => (1.0
                        - (destroy - crate::state::current_frame() as f32) / (destroy - killed))
                        .clamp(0.0, 1.0),
                    _ => 0.0,
                }
            } else {
                0.0
            };
            let _ = spring::gfx::uniform(self.sprite_death.get(), &[death_amount], 1);
        }
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::translate(position.x, position.y, position.z);
        let _ = spring::gfx::billboard(0);
        let _ = spring::gfx::tex_rect(
            -WISP_SPRITE_SIZE / 2.0,
            0.0,
            WISP_SPRITE_SIZE / 2.0,
            WISP_SPRITE_SIZE,
            0.0,
            0.0,
            1.0,
            1.0,
        );
        let _ = spring::gfx::pop_matrix(0);
    }

    /// Build the sprite shader if needed and return its id.
    fn sprite_shader_ready(&self) -> u32 {
        self.ensure_sprite_shader();
        self.sprite_shader.get()
    }

    fn ensure_sprite_shader(&self) {
        if self.sprite_shader.get() != 0 {
            return;
        }
        if let Some(id) = shader::create(WISP_SPRITE_FRAGMENT, &[]) {
            self.sprite_time.set(shader::location(id, "time"));
            self.sprite_spirit.set(shader::location(id, "spiritAmount"));
            self.sprite_unit.set(shader::location(id, "unitID"));
            self.sprite_death.set(shader::location(id, "deathAmount"));
            self.sprite_shader.set(id);
        }
    }

    fn update_and_draw_trail(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        let (spirit, wisp) = {
            let player = ctx.global().player.access();
            (player.spirit, player.wisp)
        };
        if !spirit || wisp < 0 {
            self.trail_particles.borrow_mut().clear();
            self.trail_last_position.set(None);
            return;
        }
        let Ok(position) = spring::unit_rendering::get_unit_view_position(wisp, false) else {
            self.trail_particles.borrow_mut().clear();
            self.trail_last_position.set(None);
            return;
        };

        let frame = crate::state::current_frame();
        let previous_position = self.trail_last_position.replace(Some(position));
        let (motion_x, motion_z, motion_length) = previous_position
            .map(|previous| {
                let x = position.x - previous.x;
                let z = position.z - previous.z;
                (x, z, libm::sqrtf(x * x + z * z))
            })
            .unwrap_or((0.0, 0.0, 0.0));

        // Emit from the body, rather than from the end of a ribbon. The little
        // translucent puffs inherit some backwards velocity, so they read as
        // fragments being left behind by the moving wisp. Their deterministic
        // sinusoidal offsets provide the noisy, uneven silhouette without a
        // random-number dependency in the render widget.
        {
            let mut particles = self.trail_particles.borrow_mut();
            for particle in particles.iter_mut() {
                particle.position.x += particle.velocity.x;
                particle.position.y += particle.velocity.y;
                particle.position.z += particle.velocity.z;
                particle.velocity.x *= 0.96;
                particle.velocity.y = particle.velocity.y * 0.96 + 0.02;
                particle.velocity.z *= 0.96;
                particle.life -= 1.0;
            }
            particles.retain(|particle| particle.life > 0.0);
            if motion_length > 0.25 {
                let movement_x = -motion_x / motion_length;
                let movement_z = -motion_z / motion_length;
                let emission_count = if motion_length > 2.0 { 3 } else { 2 };
                for emission in 0..emission_count {
                    if particles.len() >= MAX_TRAIL_PARTICLES {
                        particles.remove(0);
                    }
                    let seed = frame as f32 * 0.173 + emission as f32 * 2.417;
                    let side = libm::sinf(seed * 1.71) * 7.0;
                    let lift = 8.0 + libm::cosf(seed * 1.13) * 7.0;
                    particles.push(TrailParticle {
                        position: spring::Float3::new(
                            position.x + side * movement_z,
                            position.y + 13.0 + lift,
                            position.z - side * movement_x,
                        ),
                        velocity: spring::Float3::new(
                            movement_x * (0.9 + emission as f32 * 0.25)
                                + libm::cosf(seed * 2.11) * 0.45,
                            0.08 + libm::sinf(seed * 1.37).abs() * 0.12,
                            movement_z * (0.9 + emission as f32 * 0.25)
                                + libm::sinf(seed * 2.11) * 0.45,
                        ),
                        life: TRAIL_PARTICLE_LIFETIME - emission as f32 * 3.0,
                        seed,
                    });
                }
            }
        }

        // The particle puffs are the complete trail. Do not also draw the
        // older crisp ribbon/triangle strip: it gives the wisp a hard geometric
        // wake that clashes with the noisy fragments being emitted from its
        // body.
        self.draw_trail_particles(frame);
    }

    /// Draw small irregular camera-facing puffs at the particle positions.
    /// Each puff is a separate fan because the low-level shape call has no
    /// primitive-restart support. The pool stays deliberately small so this
    /// remains cheap even while the player is moving continuously.
    fn draw_trail_particles(&self, frame: i32) {
        let particles = self.trail_particles.borrow();
        if particles.is_empty() {
            return;
        }
        let matrix = self.camera_modelview.borrow();
        let right = spring::Float3::new(matrix[0], matrix[4], matrix[8]);
        let up = spring::Float3::new(matrix[1], matrix[5], matrix[9]);
        let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
            enable: true,
            set_func: false,
            func: 0,
        });
        let _ = spring::gfx::blending(true);
        let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE);
        let _ = spring::gfx::use_shader(0);

        for particle in particles.iter() {
            let fade = (particle.life / TRAIL_PARTICLE_LIFETIME).clamp(0.0, 1.0);
            let shimmer = 0.78 + 0.22 * libm::sinf(frame as f32 * 0.18 + particle.seed);
            let size = 3.5 + 3.0 * fade + 1.5 * shimmer;
            let center_alpha = 0.32 * fade * shimmer;
            let edge_alpha = 0.025 * fade;
            let mut vertices = Vec::with_capacity(12);
            vertices.push(particle_vertex(
                particle.position,
                [0.63, 0.72, 1.0, center_alpha],
            ));
            const SIDES: usize = 10;
            for side in 0..=SIDES {
                let angle = side as f32 * core::f32::consts::TAU / SIDES as f32;
                let noise = 0.72 + 0.28 * libm::sinf(particle.seed + angle * 3.0);
                let offset_x = libm::cosf(angle) * size * noise;
                let offset_y = libm::sinf(angle) * size * noise;
                let point = spring::Float3::new(
                    particle.position.x + right.x * offset_x + up.x * offset_y,
                    particle.position.y + right.y * offset_x + up.y * offset_y,
                    particle.position.z + right.z * offset_x + up.z * offset_y,
                );
                vertices.push(particle_vertex(point, [0.32, 0.43, 1.0, edge_alpha]));
            }
            let _ = spring::gfx::shape(gl::TRIANGLE_FAN, &vertices);
        }
        let _ = spring::gfx::pop_attrib(0);
    }
}

impl Widget<Global> for GfxEyesEffectWidget {
    fn name(&self) -> &'static str {
        "GfxEyesEffect"
    }

    /// Remember the camera matrices for the billboard pass.
    ///
    /// unit_wisp_draw.lua gets them with gl.LoadMatrix("camprj"/"camera");
    /// there is no named-matrix callout here, so capture them in a world callin
    /// where they are the live ones.
    fn draw_world_pre_unit(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        if let Ok(projection) = spring::gfx::get_matrix_data(gl::PROJECTION)
            && let Ok(modelview) = spring::gfx::get_matrix_data(gl::MODELVIEW)
            && projection.len() == 16
            && modelview.len() == 16
        {
            self.camera_projection
                .borrow_mut()
                .copy_from_slice(&projection);
            self.camera_modelview
                .borrow_mut()
                .copy_from_slice(&modelview);
        }
        self.update_and_draw_trail(_ctx);
    }

    fn draw_screen_effects(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        width: i32,
        height: i32,
    ) {
        let (spirit_amount, wisp, npc_wisps) = {
            let player = ctx.global().player.access();
            (
                player.form.amount(),
                player.wisp,
                ctx.global().npcs.access().wisps.clone(),
            )
        };
        if spirit_amount > 0.0 && (wisp >= 0 || !npc_wisps.is_empty()) {
            // The authored setup for DrawWisp: additive, depth-tested against
            // the world so the glow is hidden behind walls, under the camera
            // matrices rather than the screen ones this callin arrives with.
            let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
            let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
                enable: true,
                set_func: false,
                func: 0,
            });
            let _ = spring::gfx::blending(true);
            let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE);
            let _ = spring::gfx::matrix_mode(gl::PROJECTION);
            let _ = spring::gfx::push_matrix(0);
            let _ = spring::gfx::load_matrix(self.camera_projection.borrow().as_slice());
            let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
            let _ = spring::gfx::push_matrix(0);
            let _ = spring::gfx::load_matrix(self.camera_modelview.borrow().as_slice());
            let _ = spring::gfx::use_shader(self.sprite_shader_ready());

            if wisp >= 0 {
                self.draw_wisp_sprite(wisp, spirit_amount);
            }
            for unit_id in npc_wisps {
                self.draw_wisp_sprite(unit_id, spirit_amount);
            }

            let _ = spring::gfx::use_shader(0);
            let _ = spring::gfx::matrix_mode(gl::PROJECTION);
            let _ = spring::gfx::pop_matrix(0);
            let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
            let _ = spring::gfx::pop_matrix(0);
            let _ = spring::gfx::pop_attrib(0);
        }
        self.draw_eyes_effect(ctx, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::particle_vertex;

    #[test]
    fn particle_vertices_have_fixed_width_shape_attributes() {
        let vertex = particle_vertex(spring::Float3::new(1.0, 2.0, 3.0), [1.0; 4]);

        assert_eq!(vertex.vertex.len(), 3);
        assert_eq!(vertex.normal.len(), 3);
        assert_eq!(vertex.tex_coord.len(), 2);
        assert_eq!(vertex.color.len(), 4);
    }
}
