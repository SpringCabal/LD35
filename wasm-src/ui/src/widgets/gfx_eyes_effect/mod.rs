//! The authored screen treatment, spirit sprites and trail, ported from
//! `gfx_eyes_effect.lua` and `unit_wisp_draw.lua`.

use core::cell::{Cell, RefCell};

use spring::gl;
use spring_addons::ui::Widget;

use crate::state::{GAME_END_DURATION, Global, current_frame};

mod bloom;
mod npcs;
mod sprites;
mod trail;

use bloom::{Bloom, ScreenAmounts};
use npcs::Npcs;
use sprites::Sprites;
use trail::Trail;

/// The world camera, captured where it is the live one.
#[derive(Default)]
pub struct Camera {
    pub projection: [f32; 16],
    pub modelview: [f32; 16],
}

pub struct GfxEyesEffectWidget {
    camera: RefCell<Camera>,
    bloom: RefCell<Bloom>,
    npcs: RefCell<Npcs>,
    sprites: RefCell<Sprites>,
    trail: RefCell<Trail>,
    game_end_start: Cell<i32>,
}

impl Default for GfxEyesEffectWidget {
    fn default() -> Self {
        Self {
            camera: RefCell::default(),
            bloom: RefCell::default(),
            npcs: RefCell::default(),
            sprites: RefCell::default(),
            trail: RefCell::default(),
            game_end_start: Cell::new(-1),
        }
    }
}

impl GfxEyesEffectWidget {
    pub fn new() -> Self {
        Self::default()
    }

    /// The combine pass's inputs: the form transition, and the ending's fade
    /// to white and back.
    fn screen_amounts(&self, ctx: &spring_addons::AddonContext<'_, Global>) -> ScreenAmounts {
        let (form, sequence) = {
            let player = ctx.global().player.access();
            (player.form.screen_amounts(), player.game_over_sequence)
        };
        let (mut spirit, mut noise) = form;
        let frame = current_frame();
        let game_over = if !sequence {
            self.game_end_start.set(-1);
            0.0
        } else {
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
                amount = (2.0 - elapsed as f32 / GAME_END_DURATION as f32 / 2.0).clamp(0.0, 1.0);
                spirit = 2.0;
                noise = 0.0;
            }
            amount
        };
        ScreenAmounts {
            spirit,
            noise,
            game_over,
            time: frame as f32 / 30.0,
        }
    }
}

impl Widget<Global> for GfxEyesEffectWidget {
    fn name(&self) -> &'static str {
        "GfxEyesEffect"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let ending = ctx.global().player.access().game_over;
        self.npcs.borrow_mut().refresh(ending);
    }

    /// Remember the camera matrices for the billboard pass, and draw the trail.
    ///
    /// unit_wisp_draw.lua gets them with gl.LoadMatrix("camprj"/"camera");
    /// there is no named-matrix callout here, so capture them in a world callin
    /// where they are the live ones.
    fn draw_world_pre_unit(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        let mut camera = self.camera.borrow_mut();
        if let Ok(projection) = spring::gfx::get_matrix_data(gl::PROJECTION)
            && let Ok(modelview) = spring::gfx::get_matrix_data(gl::MODELVIEW)
            && projection.len() == 16
            && modelview.len() == 16
        {
            camera.projection.copy_from_slice(&projection);
            camera.modelview.copy_from_slice(&modelview);
        }

        let (spirit, wisp) = {
            let player = ctx.global().player.access();
            (player.spirit, player.wisp)
        };
        let mut trail = self.trail.borrow_mut();
        let position = (spirit && wisp >= 0)
            .then(|| spring::unit_rendering::get_unit_view_position(wisp, false).ok())
            .flatten();
        let Some(position) = position else {
            trail.clear();
            return;
        };
        let frame = current_frame();
        trail.update(position, frame);
        trail.draw(&camera, frame);
    }

    fn draw_screen_effects(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        width: i32,
        height: i32,
    ) {
        let (opacity, wisp) = {
            let player = ctx.global().player.access();
            (player.form.amount(), player.wisp)
        };
        {
            let npcs = self.npcs.borrow();
            if opacity > 0.0 && (wisp >= 0 || !npcs.wisps.is_empty()) {
                let wisps = (wisp >= 0)
                    .then_some(wisp)
                    .into_iter()
                    .chain(npcs.wisps.iter().copied());
                self.sprites
                    .borrow_mut()
                    .draw(&self.camera.borrow(), wisps, opacity);
            }
        }
        let amounts = self.screen_amounts(ctx);
        self.bloom.borrow_mut().draw(width, height, &amounts);
    }
}
