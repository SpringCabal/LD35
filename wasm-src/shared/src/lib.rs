//! Definitions the rules and UI modules must agree on, including the two
//! messages between them: [`Status`] (rules to UI) and [`Command`] (UI to
//! rules).

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;

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

const STATUS_PREFIX: &str = "PARTS_STATUS|";

impl Status {
    pub fn encode(&self) -> String {
        format!(
            "{STATUS_PREFIX}{}|{}|{}|{}|{}|{}",
            i32::from(self.spirit),
            i32::from(self.game_over),
            self.wisp,
            self.parts.bits(),
            self.ending_type,
            i32::from(self.sequence_complete),
        )
    }

    pub fn decode(message: &str) -> Option<Self> {
        let mut fields = message
            .strip_prefix(STATUS_PREFIX)?
            .split('|')
            .map(|field| field.parse::<i32>().ok());
        let mut next = || fields.next().flatten();
        let status = Self {
            spirit: next()? != 0,
            game_over: next()? != 0,
            wisp: next()?,
            parts: BodyParts::from_bits(u8::try_from(next()?).ok()?),
            ending_type: next()?,
            sequence_complete: next()? != 0,
        };
        next().is_none().then_some(status)
    }
}

/// A player command from the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// The held movement keys, each axis -1, 0 or 1; sent when they change.
    Move { x: i32, z: i32 },
    /// Space: leave or return to spirit form.
    SwitchForm,
    /// Pull the clicked lever.
    PullLever(i32),
}

impl Command {
    pub fn encode(&self) -> String {
        match self {
            Command::Move { x, z } => format!("movement|{x}|{z}"),
            Command::SwitchForm => String::from("switch_form"),
            Command::PullLever(unit) => format!("pull_lever|{unit}"),
        }
    }

    pub fn decode(message: &str) -> Option<Self> {
        let mut fields = message.split('|');
        let command = fields.next()?;
        let mut number = || fields.next()?.parse::<i32>().ok();
        match command {
            "movement" => Some(Command::Move {
                x: number()?.clamp(-1, 1),
                z: number()?.clamp(-1, 1),
            }),
            "switch_form" => Some(Command::SwitchForm),
            "pull_lever" => Some(Command::PullLever(number()?)),
            _ => None,
        }
    }
}
