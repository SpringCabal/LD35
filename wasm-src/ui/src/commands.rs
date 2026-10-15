use partsneeded_shared::Command;

pub fn send(command: Command) {
    springcabal::message::send_to_rules(&command.encode());
}
