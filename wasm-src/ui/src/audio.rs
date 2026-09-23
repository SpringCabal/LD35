use core::cell::{Cell, RefCell};

const FULL_MUSIC: &str = "sounds/music.ogg";
const MUFFLED_MUSIC: &str = "sounds/muffled.ogg";

#[derive(Default)]
struct AmbientState {
    initialized: bool,
    next_moan: i32,
    next_wind: i32,
    next_drip: i32,
    next_nose: i32,
    next_mouth: i32,
    sequence: u32,
}

thread_local! { static EARS_STATE: Cell<i8> = const { Cell::new(-1) }; static AMBIENT: RefCell<AmbientState> = RefCell::new(AmbientState::default()); }

/// Match snd_music.lua: the world is muffled until the player recovers ears.
pub fn update(body_mask: u8, spirit: bool, wisp: i32) {
    let has_ears = body_mask & 2 != 0;
    let next = i8::from(has_ears);
    let changed = EARS_STATE.with(|state| state.replace(next) != next);
    if !changed {
        play_ambient(body_mask, spirit, wisp);
        return;
    }
    let _ = spring::sound::stop_sound_stream(0);
    let _ = spring::sound::play_sound_stream(
        if has_ears { FULL_MUSIC } else { MUFFLED_MUSIC },
        if has_ears { 1.0 } else { 0.6 },
        false,
    );
    play_ambient(body_mask, spirit, wisp);
}

fn play_ambient(body_mask: u8, spirit: bool, wisp: i32) {
    if wisp < 0 {
        return;
    }
    let frame = crate::state::current_frame();
    let has_ears = body_mask & 2 != 0;
    let has_nose = body_mask & 16 != 0;
    let has_mouth = body_mask & 32 != 0;
    let position = spring::unit_rendering::get_unit_view_position(wisp, true).ok();
    let Some(position) = position else {
        return;
    };
    AMBIENT.with(|state| {
        let mut state = state.borrow_mut();
        if !state.initialized {
            // snd_ambient.lua establishes each random cooldown first, so the
            // level does not play every ambient sample on frame zero.
            state.initialized = true;
            state.next_moan = frame + cooldown_frames(&mut state.sequence, 5, 20);
            state.next_wind = frame + cooldown_frames(&mut state.sequence, 15, 60);
            state.next_drip = frame + cooldown_frames(&mut state.sequence, 3, 45);
            state.next_nose = frame + cooldown_frames(&mut state.sequence, 3, 6);
            state.next_mouth = frame + cooldown_frames(&mut state.sequence, 10, 35);
            return;
        }
        if frame >= state.next_moan {
            play_at(
                "sounds/moan.00.ogg",
                10.0,
                nearby_position(position, &mut state.sequence),
            );
            state.next_moan = frame + cooldown_frames(&mut state.sequence, 5, 20);
        }
        if !has_ears {
            return;
        }
        if frame >= state.next_wind {
            play_at(
                "sounds/wind.01.ogg",
                10.0,
                nearby_position(position, &mut state.sequence),
            );
            state.next_wind = frame + cooldown_frames(&mut state.sequence, 15, 60);
        }
        if frame >= state.next_drip {
            play_at(
                "sounds/drip.01.ogg",
                10.0,
                nearby_position(position, &mut state.sequence),
            );
            state.next_drip = frame + cooldown_frames(&mut state.sequence, 3, 45);
        }
        if has_nose && !spirit && frame >= state.next_nose {
            play_at("sounds/smell.00.ogg", 10.0, position);
            state.next_nose = frame + cooldown_frames(&mut state.sequence, 3, 6);
        }
        if has_mouth && !spirit && frame >= state.next_mouth {
            play_at("sounds/mouth.00.ogg", 10.0, position);
            state.next_mouth = frame + cooldown_frames(&mut state.sequence, 10, 35);
        }
    });
}

fn play_at(name: &str, volume: f32, position: spring::Float3) {
    let _ = spring::sound::play_sound_file(name, volume, position, spring::Float3::default(), 0);
}

fn cooldown_frames(sequence: &mut u32, minimum_seconds: i32, maximum_seconds: i32) -> i32 {
    *sequence = sequence.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let span = maximum_seconds - minimum_seconds + 1;
    (minimum_seconds + (*sequence % span as u32) as i32) * 30
}

fn nearby_position(origin: spring::Float3, sequence: &mut u32) -> spring::Float3 {
    *sequence = sequence.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let dx = 100.0 + (*sequence % 401) as f32;
    *sequence = sequence.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let dz = 100.0 + (*sequence % 401) as f32;
    spring::Float3::new(
        origin.x + if *sequence & 1 == 0 { dx } else { -dx },
        origin.y,
        origin.z + if *sequence & 2 == 0 { dz } else { -dz },
    )
}
