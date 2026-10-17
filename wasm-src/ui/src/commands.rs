use partsneeded_shared::Command;

pub fn send(spring: &spring::Spring<'_, spring::Ui>, command: Command) {
    springcabal::message::send_to_rules(spring, &command.encode());
}
