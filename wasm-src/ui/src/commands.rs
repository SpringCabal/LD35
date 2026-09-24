//! Commands to the rules module.

use partsneeded_shared::Command;

pub fn send(command: Command) {
    if let Err(error) = spring::send_lua_rules_msg(&command.encode()) {
        spring::log_warning!("partsneeded", "sending {command:?} failed: {error:?}");
    }
}
