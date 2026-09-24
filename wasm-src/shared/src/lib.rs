//! Definitions the rules and UI modules must agree on, including the two
//! messages between them: [`Status`] (rules to UI) and [`Command`] (UI to
//! rules).

#![no_std]

extern crate alloc;

use alloc::string::String;

use springcabal::message::{Encoder, fields};

/// Frames between two form switches.  The UI predicts the switch locally, so
/// it needs the same cooldown the rules module enforces.
pub const FORM_COOLDOWN: i32 = 60;

/// How close the player has to stand to pull a lever (ui_wisp_control.lua).
/// The UI checks it before sending a pull, and the rules again.
pub const LEVER_REACH: f32 = 100.0;

/// The six body parts the player collects.  The discriminant is the part's
/// bit in [`BodyParts`], which is also how [`Status`] sends them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyPart {
    Eyes,
    Ears,
    Arms,
    Legs,
    Nose,
    Mouth,
}

impl BodyPart {
    pub const ALL: [BodyPart; 6] = [
        BodyPart::Eyes,
        BodyPart::Ears,
        BodyPart::Arms,
        BodyPart::Legs,
        BodyPart::Nose,
        BodyPart::Mouth,
    ];

    /// The pickup's unit def name, also used in the `has_<name>` rules param.
    pub const fn name(self) -> &'static str {
        match self {
            BodyPart::Eyes => "eyes",
            BodyPart::Ears => "ears",
            BodyPart::Arms => "arms",
            BodyPart::Legs => "legs",
            BodyPart::Nose => "nose",
            BodyPart::Mouth => "mouth",
        }
    }

    pub const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of body parts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BodyParts(u8);

impl BodyParts {
    pub const NONE: BodyParts = BodyParts(0);
    pub const ALL: BodyParts = BodyParts(0b11_1111);

    pub const fn from_bits(bits: u8) -> Self {
        BodyParts(bits & Self::ALL.0)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, part: BodyPart) -> bool {
        self.0 & part.bit() != 0
    }

    pub fn insert(&mut self, part: BodyPart) {
        self.0 |= part.bit();
    }

    pub fn remove(&mut self, part: BodyPart) {
        self.0 &= !part.bit();
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn is_complete(self) -> bool {
        self.0 == Self::ALL.0
    }

    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }
}

/// game_end.lua's four endings, chosen when the last body part is picked up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    /// Ending #1: NPC wisps are still alive and none was saved or killed.
    WelcomeBack,
    /// Ending #2: every NPC wisp was saved.  The only ending with a
    /// ceremony: spirits arrive from all sides before the end screen.
    SoulsFreed,
    /// Ending #3: every NPC wisp was killed.
    SacrificeAccepted,
    /// Ending #4: anything else.
    NothingNoteworthy,
}

impl Ending {
    /// game_end.lua's choice, from its counters as they are at the pickup.
    /// A wisp being freed or killed stays alive, and uncounted, until its
    /// death animation ends (unit_wisp_death.lua), so a pickup during one
    /// counts it as alive.
    pub fn from_counts(alive_wisps: usize, saved_wisps: i32, killed_wisps: i32) -> Self {
        match (alive_wisps == 0, saved_wisps > 0, killed_wisps > 0) {
            (false, false, false) => Ending::WelcomeBack,
            (true, true, false) => Ending::SoulsFreed,
            (true, false, true) => Ending::SacrificeAccepted,
            _ => Ending::NothingNoteworthy,
        }
    }

    /// game_end.lua's `game_over_type`, 0-3.
    pub const fn number(self) -> i32 {
        self as i32
    }

    pub const fn from_number(number: i32) -> Option<Self> {
        match number {
            0 => Some(Ending::WelcomeBack),
            1 => Some(Ending::SoulsFreed),
            2 => Some(Ending::SacrificeAccepted),
            3 => Some(Ending::NothingNoteworthy),
            _ => None,
        }
    }

    /// The end screen's line (ui_endgame.lua).
    pub const fn text(self) -> &'static str {
        match self {
            Ending::WelcomeBack => "Welcome back, Master.   Ending #1",
            Ending::SoulsFreed => "The souls have been freed.   Ending #2",
            Ending::SacrificeAccepted => "Their sacrifice is accepted.   Ending #3",
            Ending::NothingNoteworthy => "You have achieved nothing noteworthy.   Ending #4",
        }
    }
}

/// What the UI is told about the player and the ending, sent whenever it
/// changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub spirit: bool,
    pub game_over: bool,
    /// The player wisp's unit id, or -1.
    pub wisp: i32,
    pub parts: BodyParts,
    /// The ending, once the game is over.
    pub ending: Option<Ending>,
    /// The ending's visual sequence is over and the end screen may show.
    pub sequence_complete: bool,
}

const STATUS_PREFIX: &str = "PARTS_STATUS";

impl Status {
    pub fn encode(&self) -> String {
        Encoder::new(STATUS_PREFIX)
            .flag(self.spirit)
            .flag(self.game_over)
            .field(self.wisp)
            .field(self.parts.bits())
            .field(self.ending.map_or(-1, Ending::number))
            .flag(self.sequence_complete)
            .finish()
    }

    pub fn decode(message: &str) -> Option<Self> {
        let mut fields = fields(message, STATUS_PREFIX)?;
        let status = Self {
            spirit: fields.flag()?,
            game_over: fields.flag()?,
            wisp: fields.i32()?,
            parts: BodyParts::from_bits(u8::try_from(fields.i32()?).ok()?),
            ending: match fields.i32()? {
                -1 => None,
                number => Some(Ending::from_number(number)?),
            },
            sequence_complete: fields.flag()?,
        };
        fields.is_done().then_some(status)
    }
}

/// A player command from the UI. Movement has its own message,
/// `springcabal_control::protocol`'s `movement|x|z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Space: leave or return to spirit form.
    SwitchForm,
    /// Pull the clicked lever.
    PullLever(i32),
    /// The UI (re)started and needs the current status: after a LuaUI reload
    /// the rules would otherwise only send it on the next change.
    Hello,
    /// The end screen's restart: start the level over (ui_endgame.lua's
    /// `luarules reload`). The rules take it only once the game is over.
    Restart,
}

const SWITCH_FORM: &str = "switch_form";
const PULL_LEVER: &str = "pull_lever";
const HELLO: &str = "parts_hello";
const RESTART: &str = "parts_restart";

impl Command {
    pub fn encode(&self) -> String {
        match self {
            Command::SwitchForm => Encoder::new(SWITCH_FORM).finish(),
            Command::PullLever(unit) => Encoder::new(PULL_LEVER).field(unit).finish(),
            Command::Hello => Encoder::new(HELLO).finish(),
            Command::Restart => Encoder::new(RESTART).finish(),
        }
    }

    pub fn decode(message: &str) -> Option<Self> {
        if fields(message, SWITCH_FORM).is_some() {
            return Some(Command::SwitchForm);
        }
        if fields(message, HELLO).is_some() {
            return Some(Command::Hello);
        }
        if fields(message, RESTART).is_some() {
            return Some(Command::Restart);
        }
        Some(Command::PullLever(fields(message, PULL_LEVER)?.i32()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trip() {
        let status = Status {
            spirit: true,
            game_over: false,
            wisp: 1234,
            parts: BodyParts::from_bits(0b101),
            ending: Some(Ending::SacrificeAccepted),
            sequence_complete: true,
        };
        assert_eq!(status.encode(), "PARTS_STATUS|1|0|1234|5|2|1");
        assert_eq!(Status::decode(&status.encode()), Some(status));
        assert_eq!(Status::decode("PARTS_STATUS|1|0|1234|5|2|1|9"), None);
        assert_eq!(Status::decode("PARTS_STATUS|1|0|1234|5|4|1"), None);
        let playing = Status {
            ending: None,
            ..status
        };
        assert_eq!(Status::decode(&playing.encode()), Some(playing));
    }

    #[test]
    fn ending_from_counts() {
        use Ending::*;
        assert_eq!(Ending::from_counts(10, 0, 0), WelcomeBack);
        assert_eq!(Ending::from_counts(0, 3, 0), SoulsFreed);
        assert_eq!(Ending::from_counts(0, 0, 4), SacrificeAccepted);
        assert_eq!(Ending::from_counts(0, 3, 4), NothingNoteworthy);
        assert_eq!(Ending::from_counts(3, 7, 0), NothingNoteworthy);
        assert_eq!(Ending::from_counts(0, 0, 0), NothingNoteworthy);
        // Three rescues still in their death animation: alive and not yet
        // saved, so game_end.lua welcomes the master back.
        assert_eq!(Ending::from_counts(3, 0, 0), WelcomeBack);
        for ending in [
            WelcomeBack,
            SoulsFreed,
            SacrificeAccepted,
            NothingNoteworthy,
        ] {
            assert_eq!(Ending::from_number(ending.number()), Some(ending));
        }
    }

    #[test]
    fn body_parts_insert_remove() {
        let mut parts = BodyParts::ALL;
        parts.remove(BodyPart::Mouth);
        assert!(!parts.contains(BodyPart::Mouth) && !parts.is_complete());
        assert_eq!(parts.len(), 5);
        parts.insert(BodyPart::Mouth);
        assert!(parts.is_complete());
    }

    #[test]
    fn command_round_trip() {
        for command in [
            Command::SwitchForm,
            Command::PullLever(77),
            Command::Hello,
            Command::Restart,
        ] {
            assert_eq!(Command::decode(&command.encode()), Some(command));
        }
        assert_eq!(Command::decode("movement|1|0"), None);
        assert_eq!(Command::decode("pull_lever"), None);
        assert_eq!(Command::decode("unknown|1"), None);
    }
}
