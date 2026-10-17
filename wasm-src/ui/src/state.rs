use partsneeded_shared::{BodyParts, Ending};
use spring_addons::Resource;

pub const FORM_CHANGE_FRAMES: i32 = 15;

pub const GAME_END_DURATION: i32 = 80;

#[allow(clippy::approx_constant)]
const LUA_PI: f32 = 3.14;

pub struct Player {
    pub spirit: bool,
    pub game_over: bool,
    pub ending: Option<Ending>,
    pub game_over_sequence: bool,
    pub wisp: i32,
    pub parts: BodyParts,
    pub form: FormTransition,
    pub space_used: bool,
    pub pending_spirit: Option<bool>,
    pub pending_since: i32,
    pub rules_spirit: bool,
    pub sequence_since: i32,
}

impl Player {
    pub fn end_screen_shown(&self, spring: &spring::Spring<'_, spring::Ui>) -> bool {
        self.game_over_sequence
            && self.sequence_since >= 0
            && spring.game().current_frame().unwrap_or(0) - self.sequence_since
                > GAME_END_DURATION * 2
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            spirit: true,
            game_over: false,
            ending: None,
            game_over_sequence: false,
            wisp: -1,
            parts: BodyParts::NONE,
            form: FormTransition::default(),
            space_used: false,
            pending_spirit: None,
            pending_since: 0,
            rules_spirit: true,
            sequence_since: -1,
        }
    }
}

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
    pub fn begin(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        from_spirit: bool,
        to_spirit: bool,
    ) {
        if from_spirit == to_spirit {
            return;
        }
        self.start = spring.game().current_frame().unwrap_or(0);
        self.from = f32::from(from_spirit);
        self.to = f32::from(to_spirit);
        self.duration = if from_spirit { 3 } else { FORM_CHANGE_FRAMES };
    }

    pub fn amount(&self, spring: &spring::Spring<'_, spring::Ui>) -> f32 {
        let progress = ((spring.game().current_frame().unwrap_or(0) - self.start) as f32
            / self.duration.max(1) as f32)
            .clamp(0.0, 1.0);
        self.from + (self.to - self.from) * progress
    }

    pub fn screen_amounts(&self, spring: &spring::Spring<'_, spring::Ui>) -> (f32, f32) {
        let elapsed = spring.game().current_frame().unwrap_or(0) - self.start;
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

#[derive(Default)]
pub struct Global {
    pub player: Resource<Player>,
    pub hovered_lever: Resource<Option<i32>>,
}
