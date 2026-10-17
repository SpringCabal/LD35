use alloc::string::String;
use spring::gl;

use crate::shader;

const EYES_EFFECT_FRAGMENT: &str = include_str!("../../shaders/eyes_effect.frag");
const BLOOM_BRIGHT_FRAGMENT: &str = include_str!("../../shaders/bloom_bright.frag");
const BLOOM_BLUR_H_FRAGMENT: &str = include_str!("../../shaders/bloom_blur_h.frag");
const BLOOM_BLUR_V_FRAGMENT: &str = include_str!("../../shaders/bloom_blur_v.frag");

const MAX_BRIGHTNESS: f32 = 0.22;
const ILLUM_THRESHOLD: f32 = 0.33;
const BLUR_PASSES: i32 = 2;

pub struct ScreenAmounts {
    pub spirit: f32,
    pub noise: f32,
    pub game_over: f32,
    pub time: f32,
}

#[derive(Default)]
pub struct Bloom {
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

fn blit_pass(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().matrix_mode(gl::TEXTURE);
    let _ = spring.gfx().load_identity();
    let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
    let _ = spring
        .gfx()
        .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
}

fn create_texture(
    spring: &spring::Spring<'_, spring::Ui>,
    width: i32,
    height: i32,
    format: u32,
    fbo: bool,
) -> String {
    spring
        .gfx()
        .create_texture(
            width,
            height,
            0,
            spring::gfx::GfxTextureParams {
                target: gl::TEXTURE_2D,
                format,
                border: 0,
                min_filter: gl::LINEAR,
                mag_filter: gl::LINEAR,
                wrap_s: gl::CLAMP,
                wrap_t: gl::CLAMP,
                fbo,
                ..Default::default()
            },
        )
        .unwrap_or_default()
}

impl Bloom {
    fn ensure(&mut self, spring: &spring::Spring<'_, spring::Ui>, width: i32, height: i32) {
        if self.width != width || self.height != height {
            if !self.screen_tex.is_empty() {
                let _ = spring.gfx().delete_texture(&self.screen_tex);
            }
            for texture in [&self.bright_tex_1, &self.bright_tex_2] {
                if !texture.is_empty() {
                    let _ = spring.gfx().delete_texture_fbo(texture);
                }
            }
            self.width = width;
            self.height = height;
            self.screen_tex = create_texture(spring, width, height, 0, false);
            self.bright_tex_1 = create_texture(spring, width / 4, height / 4, gl::RGB16F, true);
            self.bright_tex_2 = create_texture(spring, width / 4, height / 4, gl::RGB16F, true);
        }

        if self.initialized {
            return;
        }
        self.initialized = true;
        if let Some(id) = shader::create(spring, BLOOM_BRIGHT_FRAGMENT, &["texture0"]) {
            self.bright_shader = id;
            self.loc_bright_illum = shader::location(spring, id, "illuminationThreshold");
            self.loc_bright_inv_rx = shader::location(spring, id, "inverseRX");
            self.loc_bright_inv_ry = shader::location(spring, id, "inverseRY");
        }
        if let Some(id) = shader::create(spring, BLOOM_BLUR_H_FRAGMENT, &["texture0"]) {
            self.blur_h_shader = id;
            self.loc_blur_h_inv_rx = shader::location(spring, id, "inverseRX");
            self.loc_blur_h_radius = shader::location(spring, id, "fragKernelRadius");
        }
        if let Some(id) = shader::create(spring, BLOOM_BLUR_V_FRAGMENT, &["texture0"]) {
            self.blur_v_shader = id;
            self.loc_blur_v_inv_ry = shader::location(spring, id, "inverseRY");
            self.loc_blur_v_radius = shader::location(spring, id, "fragKernelRadius");
        }
        if let Some(id) = shader::create(spring, EYES_EFFECT_FRAGMENT, &["texture0", "texture1"]) {
            self.combine_shader = id;
            self.loc_combine_illum = shader::location(spring, id, "illuminationThreshold");
            self.loc_combine_max = shader::location(spring, id, "fragMaxBrightness");
            self.loc_combine_time = shader::location(spring, id, "time");
            self.loc_combine_spirit = shader::location(spring, id, "spiritAmount");
            self.loc_combine_noise = shader::location(spring, id, "noiseAmount");
            self.loc_combine_game_over = shader::location(spring, id, "gameOverAmount");
            self.loc_combine_debug = shader::location(spring, id, "debugDraw");
        }
    }

    pub fn draw(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        width: i32,
        height: i32,
        amounts: &ScreenAmounts,
    ) {
        if width <= 1 || height <= 1 {
            return;
        }
        self.ensure(spring, width, height);
        if self.screen_tex.is_empty() || self.combine_shader == 0 {
            return;
        }
        let screen_tex = &self.screen_tex;
        if spring
            .gfx()
            .copy_to_texture(screen_tex, 0, 0, 0, 0, width, height, gl::TEXTURE_2D, 0)
            .is_err()
        {
            return;
        }

        let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
            enable: false,
            set_func: false,
            func: 0,
        });
        let _ = spring.gfx().depth_mask(false);
        let _ = spring.gfx().blending(true);
        let _ = spring
            .gfx()
            .blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
        let _ = spring.gfx().matrix_mode(gl::PROJECTION);
        let _ = spring.gfx().push_matrix();
        let _ = spring.gfx().load_identity();
        let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
        let _ = spring.gfx().push_matrix();
        let _ = spring.gfx().load_identity();
        let _ = spring.gfx().matrix_mode(gl::TEXTURE);
        let _ = spring.gfx().load_identity();

        let bloomed = !self.bright_tex_1.is_empty()
            && !self.bright_tex_2.is_empty()
            && self.bright_shader != 0
            && self.blur_h_shader != 0
            && self.blur_v_shader != 0;
        if bloomed {
            let inverse_rx = 1.0 / width as f32;
            let inverse_ry = 1.0 / height as f32;
            let kernel_radius = height as f32 / 36.0;

            let _ = spring.gfx().use_shader(self.bright_shader);
            let _ = spring
                .gfx()
                .uniform(self.loc_bright_inv_rx, &[inverse_rx], 1);
            let _ = spring
                .gfx()
                .uniform(self.loc_bright_inv_ry, &[inverse_ry], 1);
            let _ = spring
                .gfx()
                .uniform(self.loc_bright_illum, &[ILLUM_THRESHOLD], 1);
            let _ = spring.gfx().bind_texture(screen_tex, 0, true);
            let _ = spring
                .gfx()
                .render_to_texture(&self.bright_tex_1, |spring| blit_pass(spring));

            for _ in 0..BLUR_PASSES {
                let _ = spring.gfx().use_shader(self.blur_h_shader);
                let _ = spring
                    .gfx()
                    .uniform(self.loc_blur_h_inv_rx, &[inverse_rx], 1);
                let _ = spring
                    .gfx()
                    .uniform(self.loc_blur_h_radius, &[kernel_radius], 1);
                let _ = spring.gfx().bind_texture(&self.bright_tex_1, 0, true);
                let _ = spring
                    .gfx()
                    .render_to_texture(&self.bright_tex_2, |spring| blit_pass(spring));

                let _ = spring.gfx().use_shader(self.blur_v_shader);
                let _ = spring
                    .gfx()
                    .uniform(self.loc_blur_v_inv_ry, &[inverse_ry], 1);
                let _ = spring
                    .gfx()
                    .uniform(self.loc_blur_v_radius, &[kernel_radius], 1);
                let _ = spring.gfx().bind_texture(&self.bright_tex_2, 0, true);
                let _ = spring
                    .gfx()
                    .render_to_texture(&self.bright_tex_1, |spring| blit_pass(spring));
            }
        }

        let _ = spring.gfx().use_shader(self.combine_shader);
        let _ = spring.gfx().uniform_int(self.loc_combine_debug, &[0], 1);
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_illum, &[ILLUM_THRESHOLD], 1);
        let max_brightness = if bloomed { MAX_BRIGHTNESS } else { 0.0 };
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_max, &[max_brightness], 1);
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_time, &[amounts.time], 1);
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_spirit, &[amounts.spirit], 1);
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_noise, &[amounts.noise], 1);
        let _ = spring
            .gfx()
            .uniform(self.loc_combine_game_over, &[amounts.game_over], 1);
        let _ = spring.gfx().bind_texture(screen_tex, 0, true);
        if bloomed {
            let _ = spring.gfx().bind_texture(&self.bright_tex_1, 1, true);
        }
        let _ = spring
            .gfx()
            .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
        if bloomed {
            let _ = spring.gfx().bind_texture(&self.bright_tex_1, 1, false);
        }
        let _ = spring.gfx().bind_texture(screen_tex, 0, false);
        let _ = spring.gfx().use_shader(0);
        let _ = spring.gfx().matrix_mode(gl::PROJECTION);
        let _ = spring.gfx().pop_matrix();
        let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
        let _ = spring.gfx().pop_matrix();
        let _ = spring.gfx().pop_attrib();
    }
}
