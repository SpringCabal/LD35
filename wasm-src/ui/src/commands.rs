//! Commands to the rules module.

use partsneeded_shared::Command;

/// Send `command` to the rules; a failure is logged.
pub fn send(command: Command) {
    springcabal::message::send_to_rules(&command.encode());
}
