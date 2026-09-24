//! The music and ambient sounds, ported from snd_music.lua and
//! snd_ambient.lua.

use alloc::vec::Vec;

use partsneeded_shared::{BodyPart, BodyParts};
use spring::random::unsynced as random;

const FULL_MUSIC: &str = "sounds/music.ogg";
const MUFFLED_MUSIC: &str = "sounds/muffled.ogg";

/// One of snd_ambient.lua's ambient sounds.
struct AmbientSound {
    file: &'static str,
    /// Seconds between two plays, drawn uniformly from this range.
    cooldown: (f32, f32),
    volume: (f32, f32),
    /// Only plays while the player has this part and a body; at the player's
    /// position rather than somewhere around it.
    needs_part: Option<BodyPart>,
}

const AMBIENT_SOUNDS: [AmbientSound; 5] = [
    AmbientSound {
        file: "sounds/moan.00.ogg",
        cooldown: (5.0, 20.0),
        volume: (1.0, 5.0),
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/wind.01.ogg",
        cooldown: (15.0, 60.0),
        volume: (1.0, 5.0),
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/drip.01.ogg",
        cooldown: (3.0, 45.0),
        volume: (1.0, 5.0),
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/smell.00.ogg",
        cooldown: (3.0, 6.0),
        volume: (1.0, 5.0),
        needs_part: Some(BodyPart::Nose),
    },
    AmbientSound {
        file: "sounds/mouth.00.ogg",
        cooldown: (10.0, 35.0),
        volume: (0.05, 0.2),
        needs_part: Some(BodyPart::Mouth),
    },
];

/// Where around the player the ambient sounds play, per axis.
const MIN_COORD_DISTANCE: f32 = 100.0;
const MAX_COORD_DISTANCE: f32 = 500.0;

/// Everything the sound widget remembers between updates.
#[derive(Default)]
pub struct Audio {
    /// Whether the unmuffled music is playing; `None` before the first update.
    ears: Option<bool>,
    /// The frame each ambient sound may play next; empty until the first
    /// update draws their cooldowns.
    next_play: Vec<i32>,
}

/// Match snd_music.lua: the world is muffled until the player recovers ears.
pub fn update(audio: &mut Audio, parts: BodyParts, spirit: bool, wisp: i32) {
    let has_ears = parts.contains(BodyPart::Ears);
    if audio.ears.replace(has_ears) != Some(has_ears) {
        let _ = spring::sound::stop_sound_stream(0);
        let _ = spring::sound::play_sound_stream(
            if has_ears { FULL_MUSIC } else { MUFFLED_MUSIC },
            if has_ears { 1.0 } else { 0.6 },
            false,
        );
    }
    play_ambient(audio, parts, spirit, wisp);
}

/// snd_ambient.lua: the moan always, the rest once the player can hear.
fn play_ambient(audio: &mut Audio, parts: BodyParts, spirit: bool, wisp: i32) {
    if wisp < 0 {
        return;
    }
    let Ok(position) = spring::unit_rendering::get_unit_view_position(wisp, true) else {
        return;
    };
    let frame = crate::state::current_frame();
    if audio.next_play.is_empty() {
        // Each cooldown is drawn first, so the level does not play every
        // sample on its first frame.
        audio.next_play = AMBIENT_SOUNDS
            .iter()
            .map(|sound| frame + cooldown_frames(sound))
            .collect();
        return;
    }
    let has_ears = parts.contains(BodyPart::Ears);
    for (index, sound) in AMBIENT_SOUNDS.iter().enumerate() {
        if (index > 0 && !has_ears) || frame < audio.next_play[index] {
            continue;
        }
        audio.next_play[index] = frame + cooldown_frames(sound);
        let volume = random::range(sound.volume.0, sound.volume.1) * 10.0;
        match sound.needs_part {
            None => play_at(sound.file, volume, nearby_position(position)),
            Some(part) if parts.contains(part) && !spirit => play_at(sound.file, volume, position),
            Some(_) => {}
        }
    }
}

fn play_at(name: &str, volume: f32, position: spring::Float3) {
    let _ = spring::sound::play_sound_file(name, volume, position, spring::Float3::default(), 0);
}

fn cooldown_frames(sound: &AmbientSound) -> i32 {
    (random::range(sound.cooldown.0, sound.cooldown.1) * 30.0) as i32
}

/// An offset of 100-500 elmos on each axis, either way.
fn axis_offset() -> f32 {
    let offset = 2.0 * (0.5 - random::float()) * (MAX_COORD_DISTANCE - MIN_COORD_DISTANCE);
    if offset > 0.0 {
        offset + MIN_COORD_DISTANCE
    } else {
        offset - MIN_COORD_DISTANCE
    }
}

fn nearby_position(origin: spring::Float3) -> spring::Float3 {
    spring::Float3::new(origin.x + axis_offset(), origin.y, origin.z + axis_offset())
}
