extern crate alloc;

pub mod cages;
pub mod defs;
pub mod endgame;
pub mod gadgets;
pub mod input;
#[cfg(feature = "headless-integration")]
pub mod integration;
pub mod level;
pub mod log;
pub mod sound;
pub mod state;
pub mod triggers;
pub mod unit_scripts;
pub mod units;
pub mod wisp;

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 4096,
}

spring::export_core_cus!(unit_scripts::PartsCusModule);
