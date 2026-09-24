use alloc::vec::Vec;

use partsneeded_shared::BodyParts;
use spring_addons::Resource;

pub const FORM_CHANGE_FRAMES: i32 = 15;

/// gfx_eyes_effect.lua's GAME_END_DURATION: the ending fades the screen for
/// this many frames, holds it as long again, then shows the end screen.
pub const GAME_END_DURATION: i32 = 80;

/// gfx_eyes_effect.lua writes `3.14` rather than `math.pi` in its transition
/// curves.  Substituting the real constant would change the authored shape of
/// the noise burst, so keep the literal the widget actually uses.
#[allow(clippy::approx_constant)]
const LUA_PI: f32 = 3.14;

/// What the rules module last told the UI about the player wisp.
pub struct Player {
    pub spirit: bool,
    pub game_over: bool,
    pub ending_type: i32,
    pub game_over_sequence: bool,
    pub wisp: i32,
    pub parts: BodyParts,
    pub form: FormTransition,
    /// Set once the player first presses space.  ui_wisp_control.lua shows its
    /// "Press Space" prompt as a one-shot hint that the first press dismisses,
    /// not for as long as you happen to be an eyed spirit.
    pub space_used: bool,
    /// The form we asked the rules module for, and when we asked.
    ///
    /// The synced snapshot takes a few frames to come back, and until it does
    /// it still reports the old form.  Without this, applying every snapshot
    /// blindly undid the local prediction: pressing space put you back into
    /// wisp form, re-hid the body, and -- because lever clicks are refused
    /// while you are a spirit -- ate the click as well.
    pub pending_spirit: Option<bool>,
    pub pending_since: i32,
}

/// NPC wisps, refreshed periodically rather than every frame.
#[derive(Default)]
pub struct Npcs {
    pub wisps: Vec<i32>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            spirit: true,
            game_over: false,
            ending_type: -1,
            game_over_sequence: false,
            wisp: -1,
            parts: BodyParts::NONE,
            form: FormTransition::default(),
            space_used: false,
            pending_spirit: None,
            pending_since: 0,
        }
    }
}

/// The spirit/body cross-fade, ported from `unit_wisp_draw.lua`.
pub struct FormTransition {
    pub start: i32,
    pub from: f32,
    pub to: f32,
    pub duration: i32,
}

impl Default for FormTransition {
    fn default() -> Self {
        Self {
            start: -1_000,
            from: 1.0,
            to: 1.0,
            duration: 1,
        }
    }
}

impl FormTransition {
    pub fn begin(&mut self, from_spirit: bool, to_spirit: bool) {
        if from_spirit == to_spirit {
            return;
        }
        // unit_wisp_draw.lua uses CHANGE_DURATION=15. Its spirit-to-body branch
        // multiplies progress by five, so the visible fade completes in three
        // frames; body-to-spirit takes the full authored fifteen frames.
        self.start = current_frame();
        self.from = f32::from(from_spirit);
        self.to = f32::from(to_spirit);
        self.duration = if from_spirit { 3 } else { FORM_CHANGE_FRAMES };
    }

    pub fn amount(&self) -> f32 {
        let progress =
            ((current_frame() - self.start) as f32 / self.duration.max(1) as f32).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * progress
    }

    /// The `spiritAmount`/`noiseAmount` pair the screen treatment wants.
    ///
    /// gfx_eyes_effect.lua runs its own transition off the same event but on
    /// its own timing: the full fifteen frames in *both* directions, and a
    /// noise burst that peaks well above 1 while the form is changing.  Reusing
    /// `amount()` here -- which is the sprite's three-frame exit -- skipped that
    /// burst entirely, so leaving spirit form was a silent cut rather than the
    /// authored blast of grain.
    pub fn screen_amounts(&self) -> (f32, f32) {
        let elapsed = current_frame() - self.start;
        let entering = self.to > self.from;
        if !(0..=FORM_CHANGE_FRAMES).contains(&elapsed) {
            let steady = if self.to > 0.5 { 1.0 } else { 0.0 };
            return (steady, steady);
        }
        let progress = (elapsed as f32 / FORM_CHANGE_FRAMES as f32).clamp(0.0, 1.0);
        if entering {
            (progress, progress + libm::sinf(progress * LUA_PI / 1.5))
        } else {
            (
                1.0 - progress,
                1.0 - progress + libm::cosf(progress * LUA_PI / 2.0) * 3.0,
            )
        }
    }
}

pub fn current_frame() -> i32 {
    spring::game::frame().map_or(0, |frame| frame as i32)
}

/// The shared communication namespace for the UI module.
#[derive(Default)]
pub struct Global {
    pub player: Resource<Player>,
    pub npcs: Resource<Npcs>,
    /// The lever the highlight glows on as of the last draw pass.
    ///
    /// The click handler uses it instead of tracing again: two TraceScreenRay
    /// calls in one frame interfere, and the second returns no hit.  It also
    /// guarantees a click lands on whatever is actually glowing.
    pub hovered_lever: Resource<Option<i32>>,
}
