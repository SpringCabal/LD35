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

/// What the UI is told about the player and the ending, sent whenever it
/// changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub spirit: bool,
    pub game_over: bool,
    /// The player wisp's unit id, or -1.
    pub wisp: i32,
    pub parts: BodyParts,
    /// game_end.lua's ending, 0-3; meaningful once `game_over` is set.
    pub ending_type: i32,
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
            .field(self.ending_type)
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
            ending_type: fields.i32()?,
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
}

const SWITCH_FORM: &str = "switch_form";
const PULL_LEVER: &str = "pull_lever";
const HELLO: &str = "parts_hello";

impl Command {
    pub fn encode(&self) -> String {
        match self {
            Command::SwitchForm => Encoder::new(SWITCH_FORM).finish(),
            Command::PullLever(unit) => Encoder::new(PULL_LEVER).field(unit).finish(),
            Command::Hello => Encoder::new(HELLO).finish(),
        }
    }

    pub fn decode(message: &str) -> Option<Self> {
        if fields(message, SWITCH_FORM).is_some() {
            return Some(Command::SwitchForm);
        }
        if fields(message, HELLO).is_some() {
            return Some(Command::Hello);
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
            ending_type: 2,
            sequence_complete: true,
        };
        assert_eq!(status.encode(), "PARTS_STATUS|1|0|1234|5|2|1");
        assert_eq!(Status::decode(&status.encode()), Some(status));
        assert_eq!(Status::decode("PARTS_STATUS|1|0|1234|5|2|1|9"), None);
    }

    #[test]
    fn command_round_trip() {
        for command in [Command::SwitchForm, Command::PullLever(77), Command::Hello] {
            assert_eq!(Command::decode(&command.encode()), Some(command));
        }
        assert_eq!(Command::decode("movement|1|0"), None);
        assert_eq!(Command::decode("pull_lever"), None);
        assert_eq!(Command::decode("unknown|1"), None);
    }
}
