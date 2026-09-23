//! Definitions the rules and UI modules must agree on.

#![no_std]

/// Frames between two form switches.  The UI predicts the switch locally, so
/// it needs the same cooldown the rules module enforces.
pub const FORM_COOLDOWN: i32 = 60;

/// The six body parts the player collects.  The discriminant is the part's
/// bit in [`BodyParts`], which is also how the status message sends them.
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
