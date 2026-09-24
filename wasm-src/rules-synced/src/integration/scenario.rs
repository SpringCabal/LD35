//! The level units the scenario uses.

/// The lever that opens cage 23261; the scenario pulls it twice.
pub const TEST_LEVER: i32 = 14725;
/// The cage the test lever opens.  Kill plate 26492 links to it too.
pub const TEST_LEVER_CAGE: i32 = 23261;
/// The levers that open the first gate (20): the three the floor markers
/// in its room circle (gfx_blood.lua), read from game_custom_spawn.lua's
/// `["20"] = {-29318, 23884, -14619, -19115}`. Kept apart from the rules'
/// table so a mistake there can't make the check pass.
pub const LEVER_GATE_PULLS: [i32; 3] = [29318, 14619, 19115];
/// A standing plate linked to two cages; the embodied player with legs stands
/// on it.
pub const TEST_PLATE: i32 = 12701;
/// How many commands and movement messages the scenario sends.
pub const SCENARIO_MESSAGES: u32 = 13;
/// The last room's one-way plate, which opens gate 23.
pub const LAST_ROOM_PLATE: i32 = 11688;
/// A plate that kills the prisoners of four cages.
pub const KILL_PLATE: i32 = 12688;
/// How many prisoners the kill plate kills.
pub const KILL_PLATE_CAGES: usize = 4;
