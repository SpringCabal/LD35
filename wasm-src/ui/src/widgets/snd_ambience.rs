use core::cell::Cell;
use partsneeded_shared::BodyPart;

use spring_addons::ui::Widget;

use crate::state::Global;

#[derive(Default)]
pub struct SndAmbienceWidget {
    next_play: Cell<Option<[i32; AMBIENT_SOUNDS.len()]>>,
}

impl SndAmbienceWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for SndAmbienceWidget {
    fn name(&self) -> &'static str {
        "SndAmbience"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        let (parts, spirit, wisp) = {
            let player = ctx.global().player.access();
            (player.parts, player.spirit, player.wisp)
        };
        if wisp < 0 {
            return;
        }
        let Ok(position) = spring.unit_rendering().get_unit_view_position(wisp, true) else {
            return;
        };
        let frame = spring.game().current_frame().unwrap_or(0);
        let Some(mut next_play) = self.next_play.get() else {
            self.next_play.set(Some(
                AMBIENT_SOUNDS.map(|sound| frame + cooldown_frames(spring, &sound)),
            ));
            return;
        };
        let has_ears = parts.contains(BodyPart::Ears);
        for (sound, next_frame) in AMBIENT_SOUNDS.iter().zip(&mut next_play) {
            if (sound.needs_ears && !has_ears) || frame < *next_frame {
                continue;
            }
            *next_frame = frame + cooldown_frames(spring, sound);
            let volume = spring
                .unsynced_random()
                .range(sound.volume.0, sound.volume.1)
                .expect("unsynced random")
                * 10.0;
            match sound.needs_part {
                None => play_at(
                    spring,
                    sound.file,
                    volume,
                    nearby_position(spring, position),
                ),
                Some(part) if parts.contains(part) && !spirit => {
                    play_at(spring, sound.file, volume, position)
                }
                Some(_) => {}
            }
        }
        self.next_play.set(Some(next_play));
    }
}

struct AmbientSound {
    file: &'static str,
    cooldown: (f32, f32),
    volume: (f32, f32),
    needs_ears: bool,
    needs_part: Option<BodyPart>,
}

const AMBIENT_SOUNDS: [AmbientSound; 5] = [
    AmbientSound {
        file: "sounds/moan.00.ogg",
        cooldown: (5.0, 20.0),
        volume: (1.0, 5.0),
        needs_ears: false,
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/wind.01.ogg",
        cooldown: (15.0, 60.0),
        volume: (1.0, 5.0),
        needs_ears: true,
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/drip.01.ogg",
        cooldown: (3.0, 45.0),
        volume: (1.0, 5.0),
        needs_ears: true,
        needs_part: None,
    },
    AmbientSound {
        file: "sounds/smell.00.ogg",
        cooldown: (3.0, 6.0),
        volume: (1.0, 5.0),
        needs_ears: true,
        needs_part: Some(BodyPart::Nose),
    },
    AmbientSound {
        file: "sounds/mouth.00.ogg",
        cooldown: (10.0, 35.0),
        volume: (0.05, 0.2),
        needs_ears: true,
        needs_part: Some(BodyPart::Mouth),
    },
];

const MIN_COORD_DISTANCE: f32 = 100.0;
const MAX_COORD_DISTANCE: f32 = 500.0;

fn play_at(
    spring: &spring::Spring<'_, spring::Ui>,
    name: &str,
    volume: f32,
    position: spring::Float3,
) {
    let _ = spring
        .sound()
        .play_sound_file(name, volume, position, spring::Float3::default(), 0);
}

fn cooldown_frames(spring: &spring::Spring<'_, spring::Ui>, sound: &AmbientSound) -> i32 {
    (spring
        .unsynced_random()
        .range(sound.cooldown.0, sound.cooldown.1)
        .expect("unsynced random")
        * spring::consts::GAME_SPEED as f32) as i32
}

fn axis_offset(spring: &spring::Spring<'_, spring::Ui>) -> f32 {
    let offset = 2.0
        * (0.5 - spring.unsynced_random().float().expect("unsynced random"))
        * (MAX_COORD_DISTANCE - MIN_COORD_DISTANCE);
    if offset > 0.0 {
        offset + MIN_COORD_DISTANCE
    } else {
        offset - MIN_COORD_DISTANCE
    }
}

fn nearby_position(
    spring: &spring::Spring<'_, spring::Ui>,
    origin: spring::Float3,
) -> spring::Float3 {
    spring::Float3::new(
        origin.x + axis_offset(spring),
        origin.y,
        origin.z + axis_offset(spring),
    )
}
