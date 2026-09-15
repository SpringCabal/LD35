extern crate alloc;

pub mod cages;
pub mod endgame;
pub mod gadgets;
pub mod input;
#[cfg(feature = "headless-integration")]
pub mod integration;
pub mod level;
pub mod sound;
pub mod state;
pub mod triggers;
pub mod wisp;

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 4096,
}
