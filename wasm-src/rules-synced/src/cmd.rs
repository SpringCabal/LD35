//! Engine command ids, until the spring crate exposes the `CMD_*` constants.

pub const STOP: i32 = 0;
pub const MOVE: i32 = 10;
/// Drives every lever, plate, gate and cage: the engine turns it into the unit
/// script's Activate/Deactivate.
pub const ONOFF: i32 = 85;
