use alloc::vec::Vec;

use partsneeded_shared::{BodyPart, BodyParts};
use spring::commands;
use springcabal_control::goal::{Bounds, Clamp, GoalHeight, MoveGoal};

use crate::state::Wisp;

const STEER: MoveGoal = MoveGoal::ahead(120.0)
    .radius(16.0)
    .raw(true)
    .height(GoalHeight::Unit)
    .bounds(Bounds::Map {
        clearance: 1.0,
        clamp: Clamp::KeepHeading,
    });

const GLIDE_SPEED: f32 = 6.0;
const GLIDE: MoveGoal = STEER.reach(20.0);

const PICKUP_RADIUS: f32 = 50.0;

pub fn move_wisp(wisp: &Wisp) -> bool {
    let Some(direction) = wisp.steering.direction() else {
        return false;
    };
    STEER.apply(wisp.id, direction).unwrap_or(false)
}

pub fn stop_wisp(id: i32) {
    commands::give_order_to_unit(id, spring::cmd::STOP, &[], 0, 0).then(move |result| {
        crate::log::check("STOP order", id, result);
        let Ok(velocity) = spring::get_unit_velocity(id) else {
            return;
        };
        let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if speed > GLIDE_SPEED {
            let _ = GLIDE.apply(id, (velocity.x, velocity.z));
        }
    });
}

pub fn cancel_upward_velocity(wisp: &Wisp) {
    if wisp.id < 0 {
        return;
    }
    let Ok(velocity) = spring::get_unit_velocity(wisp.id) else {
        return;
    };
    if velocity.y > 0.0 {
        let _ = spring::unit_control::set_unit_velocity(
            wisp.id,
            spring::Float3::new(velocity.x, 0.0, velocity.z),
        );
    }
}

pub fn collect_body_parts(wisp: &mut Wisp) -> Vec<(i32, bool, i32)> {
    let Ok(position) = spring::get_unit_position(wisp.id, false, false) else {
        return Vec::new();
    };
    let defs = crate::defs::defs();
    let mut collected = Vec::new();
    for id in &wisp.body_parts {
        let Ok(part_position) = spring::get_unit_position(*id, false, false) else {
            continue;
        };
        let dx = position.x - part_position.x;
        let dz = position.z - part_position.z;
        if dx * dx + dz * dz > PICKUP_RADIUS * PICKUP_RADIUS {
            continue;
        }
        let Some(part) = defs.body_part(crate::defs::unit_def(*id)) else {
            continue;
        };
        collected.push((*id, part));
    }
    for &(_, part) in &collected {
        if !wisp.collected_parts.contains(part) {
            wisp.collected_parts.insert(part);
            wisp.pieces_dirty = true;
        }
    }
    wisp.body_parts
        .retain(|id| !collected.iter().any(|(collected, _)| collected == id));
    collected
        .into_iter()
        .map(|(id, _)| (id, true, wisp.id))
        .collect()
}

pub fn rotate_body_parts(wisp: &Wisp) {
    for id in &wisp.body_parts {
        let Ok(rotation) = spring::rules_synced::units_info::get_unit_rotation(*id) else {
            continue;
        };
        let _ = spring::set_unit_rotation(
            *id,
            spring::Float3::new(rotation.pitch, rotation.yaw + 0.1, rotation.roll),
        );
    }
}

pub const WISP_PIECE_GROUPS: [(&str, Option<BodyPart>); 18] = [
    ("Body", None),
    ("Head", None),
    ("Eyes", Some(BodyPart::Eyes)),
    ("Ears", Some(BodyPart::Ears)),
    ("ArmL", Some(BodyPart::Arms)),
    ("ArmR", Some(BodyPart::Arms)),
    ("ForeArmL", Some(BodyPart::Arms)),
    ("ForeArmR", Some(BodyPart::Arms)),
    ("HandL", Some(BodyPart::Arms)),
    ("HandR", Some(BodyPart::Arms)),
    ("ThighL", Some(BodyPart::Legs)),
    ("ThighR", Some(BodyPart::Legs)),
    ("CrusL", Some(BodyPart::Legs)),
    ("CrusR", Some(BodyPart::Legs)),
    ("FootL", Some(BodyPart::Legs)),
    ("FootR", Some(BodyPart::Legs)),
    ("Nose", Some(BodyPart::Nose)),
    ("Mouth", Some(BodyPart::Mouth)),
];

pub fn group_visible(group: Option<BodyPart>, parts: Option<BodyParts>) -> bool {
    parts.is_some_and(|parts| group.is_none_or(|part| parts.contains(part)))
}

pub fn sync_wisp_pieces(id: i32, spirit: bool, collected_parts: BodyParts) {
    if id < 0 {
        return;
    }
    let parts = (!spirit).then_some(collected_parts);
    crate::unit_scripts::with_host(|host| {
        crate::unit_scripts::set_piece_mask(host, id, parts);
    });
}
