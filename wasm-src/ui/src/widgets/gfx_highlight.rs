//! The mouse-over glow on a reachable lever, ported from `gfx_highlight.lua`.
//!
//! Structurally this is the same three-pass chain as the cartoon outline in
//! Flove (silhouette into an FBO, separable blur, composite), but the authored
//! widget here draws exactly one unit -- the lever under the cursor -- and only
//! while an armed, embodied player is standing close enough to pull it.  That
//! glow is the game's only cue for which lever is interactive, so it is not
//! decoration.

use alloc::string::String;
use core::cell::RefCell;

use partsneeded_shared::BodyPart;
use spring::gfx::{GfxUnitDrawOptions, TextureCreateParams};
use spring::gl;
use spring_addons::event::ViewGeometry;
use spring_addons::ui::Widget;

use crate::shader;
use crate::state::Global;

const DEPTH_FRAGMENT: &str = include_str!("../shaders/highlight_depth.frag");
const BLUR_H_FRAGMENT: &str = include_str!("../shaders/highlight_blur_h.frag");
const BLUR_V_FRAGMENT: &str = include_str!("../shaders/highlight_blur_v.frag");

/// How close the player has to stand, matching gfx_highlight.lua and
/// ui_wisp_control.lua.  Both use the same radius so that everything the glow
/// offers can actually be clicked.
pub const REACH: f32 = 100.0;

struct Pipeline {
    initialized: bool,
    vsx: i32,
    vsy: i32,
    res_changed: bool,
    depth_tex: String,
    offscreen_tex: String,
    blur_tex: String,
    depth_shader: u32,
    blur_h_shader: u32,
    blur_v_shader: u32,
    loc_screen_xy: i32,
    loc_screen_x: i32,
    loc_screen_y: i32,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            initialized: false,
            vsx: 0,
            vsy: 0,
            res_changed: true,
            depth_tex: String::new(),
            offscreen_tex: String::new(),
            blur_tex: String::new(),
            depth_shader: 0,
            blur_h_shader: 0,
            blur_v_shader: 0,
            loc_screen_xy: -1,
            loc_screen_x: -1,
            loc_screen_y: -1,
        }
    }
}

/// The hovered unit, if it is a lever the player could pull right now.
fn find_hovered_lever(player_wisp: i32) -> i32 {
    let Ok(mouse) = spring::input::get_mouse_state(0) else {
        return -1;
    };
    if mouse.offscreen {
        return -1;
    }
    let Ok(trace) = spring::camera::trace_screen_ray(
        mouse.x,
        mouse.y,
        spring::camera::TraceScreenRayOptions {
            only_coords: false,
            use_minimap: false,
            include_sky: false,
            ignore_water: false,
            height_offset: 0.0,
        },
    ) else {
        return -1;
    };
    let unit_id = trace.hit_id;
    if crate::defs::unit_def(unit_id) != crate::defs::defs().lever {
        return -1;
    }
    let (Ok(lever), Ok(wisp)) = (
        spring::get_unit_position(unit_id, false, false),
        spring::get_unit_position(player_wisp, false, false),
    ) else {
        return -1;
    };
    let (dx, dz) = (lever.x - wisp.x, lever.z - wisp.z);
    if dx * dx + dz * dz >= REACH * REACH {
        return -1;
    }
    unit_id
}

fn recreate_textures(state: &mut Pipeline) {
    if !state.depth_tex.is_empty() {
        let _ = spring::gfx::delete_texture(&state.depth_tex);
        state.depth_tex.clear();
    }
    if !state.offscreen_tex.is_empty() {
        let _ = spring::gfx::delete_texture_fbo(&state.offscreen_tex);
        state.offscreen_tex.clear();
    }
    if !state.blur_tex.is_empty() {
        let _ = spring::gfx::delete_texture_fbo(&state.blur_tex);
        state.blur_tex.clear();
    }
    if state.vsx <= 1 || state.vsy <= 1 {
        return;
    }
    if let Ok(tex) = spring::gfx::create_texture(
        state.vsx,
        state.vsy,
        0,
        TextureCreateParams {
            target: gl::TEXTURE_2D,
            format: gl::DEPTH_COMPONENT24,
            border: 0,
            min_filter: gl::NEAREST,
            mag_filter: gl::NEAREST,
            ..Default::default()
        },
    ) {
        state.depth_tex = tex;
    }
    // fbo_depth matters: without a depth buffer on the silhouette target the
    // lever's own far faces draw over its near ones.
    if let Ok(tex) = spring::gfx::create_texture(
        state.vsx,
        state.vsy,
        0,
        TextureCreateParams {
            target: gl::TEXTURE_2D,
            format: 0,
            border: 0,
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
            wrap_s: gl::CLAMP,
            wrap_t: gl::CLAMP,
            fbo: true,
            fbo_depth: true,
            ..Default::default()
        },
    ) {
        state.offscreen_tex = tex;
    }
    if let Ok(tex) = spring::gfx::create_texture(
        state.vsx,
        state.vsy,
        0,
        TextureCreateParams {
            target: gl::TEXTURE_2D,
            format: 0,
            border: 0,
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
            wrap_s: gl::CLAMP,
            wrap_t: gl::CLAMP,
            fbo: true,
            ..Default::default()
        },
    ) {
        state.blur_tex = tex;
    }
    state.res_changed = true;
}

fn ensure_initialized(state: &mut Pipeline) {
    if state.initialized {
        return;
    }
    state.initialized = true;
    if let Some(id) = shader::create(DEPTH_FRAGMENT, &["tex0"]) {
        state.depth_shader = id;
        state.loc_screen_xy = shader::location(id, "screenXY");
    }
    if let Some(id) = shader::create(BLUR_H_FRAGMENT, &["tex0"]) {
        state.blur_h_shader = id;
        state.loc_screen_x = shader::location(id, "screenX");
    }
    if let Some(id) = shader::create(BLUR_V_FRAGMENT, &["tex0"]) {
        state.blur_v_shader = id;
        state.loc_screen_y = shader::location(id, "screenY");
    }
    recreate_textures(state);
}

fn update_uniforms(state: &mut Pipeline) {
    if !state.res_changed || state.vsx <= 1 || state.vsy <= 1 {
        return;
    }
    state.res_changed = false;
    if state.depth_shader != 0 && state.loc_screen_xy >= 0 {
        let _ = spring::gfx::use_shader(state.depth_shader);
        let _ = spring::gfx::uniform(
            state.loc_screen_xy,
            &[state.vsx as f32, state.vsy as f32],
            2,
        );
    }
    if state.blur_h_shader != 0 && state.loc_screen_x >= 0 {
        let _ = spring::gfx::use_shader(state.blur_h_shader);
        let _ = spring::gfx::uniform_int(state.loc_screen_x, &[state.vsx], 1);
    }
    if state.blur_v_shader != 0 && state.loc_screen_y >= 0 {
        let _ = spring::gfx::use_shader(state.blur_v_shader);
        let _ = spring::gfx::uniform_int(state.loc_screen_y, &[state.vsy], 1);
    }
    let _ = spring::gfx::use_shader(0);
}

/// Draw the lever's silhouette.  `render_to_texture` installs identity
/// matrices before invoking this, and a native module -- unlike a Lua widget
/// calling gl.ResetMatrices() -- has to put the camera back itself.
fn pass_draw_target(projection: &[f32; 16], modelview: &[f32; 16], target: i32) {
    let _ = spring::gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring::gfx::matrix_mode(gl::PROJECTION);
    let _ = spring::gfx::load_matrix(projection);
    let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
    let _ = spring::gfx::load_matrix(modelview);
    let _ = spring::gfx::unit(
        target,
        GfxUnitDrawOptions {
            apply_transform: true,
            do_raw_draw: true,
            no_lua_call: false,
            full_model: true,
            ..Default::default()
        },
    );
}

fn blur_pass(shader: u32, vsx: i32, vsy: i32) {
    if shader == 0 || vsx <= 1 || vsy <= 1 {
        return;
    }
    let _ = spring::gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring::gfx::matrix_mode(gl::TEXTURE);
    let _ = spring::gfx::load_identity(0);
    let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
    let _ = spring::gfx::use_shader(shader);
    let x_off = 0.5 / (vsx as f32);
    let y_off = 0.5 / (vsy as f32);
    let _ = spring::gfx::tex_rect(
        -1.0 - x_off,
        1.0 + y_off,
        1.0 + x_off,
        -1.0 - y_off,
        0.0,
        1.0,
        1.0,
        0.0,
    );
}

#[derive(Default)]
pub struct GfxHighlightWidget {
    pipeline: RefCell<Pipeline>,
}

impl GfxHighlightWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxHighlightWidget {
    fn name(&self) -> &'static str {
        "GfxHighlight"
    }

    fn view_resize(&self, _ctx: &spring_addons::AddonContext<'_, Global>, geometry: &ViewGeometry) {
        let mut state = self.pipeline.borrow_mut();
        state.vsx = geometry.view_size.0;
        state.vsy = geometry.view_size.1;
        if state.initialized {
            recreate_textures(&mut state);
        }
    }

    fn draw_world_pre_unit(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        // gfx_highlight.lua's own gate: spirits cannot touch anything, and
        // without arms there is nothing to pull with.
        let (spirit, has_arms, wisp, game_over) = {
            let player = ctx.global().player.access();
            (
                player.spirit,
                player.parts.contains(BodyPart::Arms),
                player.wisp,
                player.game_over,
            )
        };
        let target = if spirit || !has_arms || game_over || wisp < 0 {
            -1
        } else {
            find_hovered_lever(wisp)
        };
        *ctx.global().hovered_lever.access_mut() = (target >= 0).then_some(target);
        if target < 0 {
            return;
        }

        let (depth_tex, offscreen_tex, blur_tex, depth_shader, blur_h, blur_v, vsx, vsy) = {
            let mut state = self.pipeline.borrow_mut();
            ensure_initialized(&mut state);
            if let Ok(geometry) = spring::display::get_view_geometry(0)
                && geometry.view_size_x > 1
                && geometry.view_size_y > 1
                && (geometry.view_size_x != state.vsx || geometry.view_size_y != state.vsy)
            {
                state.vsx = geometry.view_size_x;
                state.vsy = geometry.view_size_y;
                recreate_textures(&mut state);
            }
            update_uniforms(&mut state);
            (
                state.depth_tex.clone(),
                state.offscreen_tex.clone(),
                state.blur_tex.clone(),
                state.depth_shader,
                state.blur_h_shader,
                state.blur_v_shader,
                state.vsx,
                state.vsy,
            )
        };
        if depth_tex.is_empty()
            || offscreen_tex.is_empty()
            || blur_tex.is_empty()
            || depth_shader == 0
            || vsx <= 1
            || vsy <= 1
        {
            return;
        }

        let _ = spring::gfx::copy_to_texture(&depth_tex, 0, 0, 0, 0, vsx, vsy, gl::TEXTURE_2D, 0);
        let _ = spring::gfx::bind_texture(&depth_tex, 0, true);

        // The world matrices, restored inside the silhouette pass.
        let mut projection = [0.0; 16];
        let mut modelview = [0.0; 16];
        if let Ok(live_projection) = spring::gfx::get_matrix_data(gl::PROJECTION)
            && let Ok(live_modelview) = spring::gfx::get_matrix_data(gl::MODELVIEW)
            && live_projection.len() == 16
            && live_modelview.len() == 16
        {
            projection.copy_from_slice(&live_projection);
            modelview.copy_from_slice(&live_modelview);
        }

        let _ = spring::gfx::use_shader(depth_shader);
        let _ = spring::gfx::render_to_texture(&offscreen_tex, || {
            pass_draw_target(&projection, &modelview, target);
        });

        let _ = spring::gfx::bind_texture(&offscreen_tex, 0, true);
        let _ = spring::gfx::render_to_texture(&blur_tex, || blur_pass(blur_v, vsx, vsy));
        let _ = spring::gfx::bind_texture(&blur_tex, 0, true);
        let _ = spring::gfx::render_to_texture(&offscreen_tex, || blur_pass(blur_h, vsx, vsy));

        let _ = spring::gfx::use_shader(0);
        let _ = spring::gfx::matrix_mode(gl::TEXTURE);
        let _ = spring::gfx::load_identity(0);
        let _ = spring::gfx::matrix_mode(gl::PROJECTION);
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::load_identity(0);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::load_identity(0);

        // The world pass leaves alpha and depth testing on, which would eat the
        // soft edge of the glow.  A Lua widget has these reset for it.
        let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring::gfx::alpha_test(false, gl::ALWAYS, 0.0);
        let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
            enable: false,
            set_func: false,
            func: 0,
        });
        let _ = spring::gfx::depth_mask(false);
        let _ = spring::gfx::culling(false);
        let _ = spring::gfx::blending(true);
        let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        let _ = spring::gfx::bind_texture(&offscreen_tex, 0, true);

        let x_off = 0.5 / (vsx as f32);
        let y_off = 0.5 / (vsy as f32);
        let _ = spring::gfx::tex_rect(
            -1.0 - x_off,
            1.0 + y_off,
            1.0 + x_off,
            -1.0 - y_off,
            0.0,
            1.0,
            1.0,
            0.0,
        );

        let _ = spring::gfx::bind_texture(&offscreen_tex, 0, false);
        let _ = spring::gfx::matrix_mode(gl::PROJECTION);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::use_shader(0);
        let _ = spring::gfx::pop_attrib(0);
    }
}
