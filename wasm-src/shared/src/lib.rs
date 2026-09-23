//! Definitions the rules and UI modules must agree on.

#![no_std]

/// Frames between two form switches.  The UI predicts the switch locally, so
/// it needs the same cooldown the rules module enforces.
pub const FORM_COOLDOWN: i32 = 60;
