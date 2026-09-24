//! The spirit billboards, ported from `DrawWisp` in unit_wisp_draw.lua.

use spring::gl;

use super::Camera;
use crate::shader;

const WISP_SPRITE_FRAGMENT: &str = include_str!("../../shaders/wisp_sprite.frag");

/// The wisp billboard is a 40x40 elmo quad in unit_wisp_draw.lua.
const WISP_SPRITE_SIZE: f32 = 40.0;

#[derive(Default)]
pub struct Sprites {
    shader: u32,
    loc_time: i32,
    loc_spirit: i32,
    loc_unit: i32,
    loc_death: i32,
}

impl Sprites {
    fn ensure_shader(&mut self) -> bool {
        if self.shader == 0
            && let Some(id) = shader::create(WISP_SPRITE_FRAGMENT, &[])
        {
            self.loc_time = shader::location(id, "time");
            self.loc_spirit = shader::location(id, "spiritAmount");
            self.loc_unit = shader::location(id, "unitID");
            self.loc_death = shader::location(id, "deathAmount");
            self.shader = id;
        }
        self.shader != 0
    }

    /// Draw every wisp's billboard at `opacity`: additive, depth-tested against
    /// the world so the glow is hidden behind walls, under the camera matrices
    /// rather than the screen ones the calling callin arrives with.
    pub fn draw(&mut self, camera: &Camera, wisps: impl Iterator<Item = i32>, opacity: f32) {
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
        let _ = spring::gfx::load_matrix(&camera.projection);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::push_matrix(0);
        let _ = spring::gfx::load_matrix(&camera.modelview);
        if self.ensure_shader() {
            let _ = spring::gfx::use_shader(self.shader);
            let frame = spring::game::current_frame() as f32;
            // unit_wisp_draw.lua quantises its time to a tenth of a second, so
            // the per-pixel noise resamples in visible steps instead of
            // shimmering every frame.
            let seconds = frame / spring::consts::GAME_SPEED as f32;
            let _ = spring::gfx::uniform(self.loc_time, &[(seconds * 10.0).floor() / 10.0], 1);
            let _ = spring::gfx::uniform(self.loc_spirit, &[opacity], 1);
            for unit_id in wisps {
                self.draw_one(unit_id, frame);
            }
        }
        let _ = spring::gfx::use_shader(0);
        let _ = spring::gfx::matrix_mode(gl::PROJECTION);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
        let _ = spring::gfx::pop_matrix(0);
        let _ = spring::gfx::pop_attrib(0);
    }

    fn draw_one(&self, unit_id: i32, frame: f32) {
        // unit_wisp_draw.lua calls Spring.GetUnitViewPosition(unitID) with no
        // second argument, i.e. the unit's base position.
        let Ok(position) = spring::unit_rendering::get_unit_view_position(unit_id, false) else {
            return;
        };
        let _ = spring::gfx::uniform(self.loc_unit, &[unit_id as f32], 1);
        let _ = spring::gfx::uniform(self.loc_death, &[death_amount(unit_id, frame)], 1);
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
}

/// unit_wisp_draw.lua fades every saved or killed wisp through the public
/// is_dying/killed_frame/destroy_frame rules params.  The player is never in
/// that lifecycle, so absent params give it no fade.
fn death_amount(unit_id: i32, frame: f32) -> f32 {
    if spring::get_unit_rules_param_f32(unit_id, "is_dying") != Ok(1.0) {
        return 0.0;
    }
    let destroy = spring::get_unit_rules_param_f32(unit_id, "destroy_frame");
    let killed = spring::get_unit_rules_param_f32(unit_id, "killed_frame");
    match (destroy, killed) {
        (Ok(destroy), Ok(killed)) if destroy > killed => {
            (1.0 - (destroy - frame) / (destroy - killed)).clamp(0.0, 1.0)
        }
        _ => 0.0,
    }
}
