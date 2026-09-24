use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::defs::{defs, unit_def};
use crate::state::Global;

use partsneeded_shared::{BodyPart, Command, FORM_COOLDOWN};

const KEY_A: i32 = b'a' as i32;
const KEY_D: i32 = b'd' as i32;
const KEY_S: i32 = b's' as i32;
const KEY_W: i32 = b'w' as i32;
const KEY_UP: i32 = 273;
const KEY_DOWN: i32 = 274;
const KEY_RIGHT: i32 = 275;
const KEY_LEFT: i32 = 276;

/// The clickable band around the end screen's "Click or press R to restart"
/// line, relative to the screen centre.
const RESTART_PROMPT_HALF_WIDTH: i32 = 190;
const RESTART_PROMPT_TOP: i32 = -52;
const RESTART_PROMPT_BOTTOM: i32 = -12;

fn send(command: Command) {
    if let Err(error) = spring::send_lua_rules_msg(&command.encode()) {
        spring::log_warning!("partsneeded", "sending {command:?} failed: {error:?}");
    }
}

/// Direct player control, ported from `ui_wisp_control.lua`: movement, form
/// switches, lever pulls and the end screen's restart.
#[derive(Default)]
pub struct GuiWispControlWidget {
    left: core::cell::Cell<bool>,
    right: core::cell::Cell<bool>,
    up: core::cell::Cell<bool>,
    down: core::cell::Cell<bool>,
    /// The movement the rules module was last told about.
    sent_movement: core::cell::Cell<(i32, i32)>,
    last_form_change: core::cell::Cell<i32>,
    screen_width: core::cell::Cell<i32>,
    screen_height: core::cell::Cell<i32>,
}

impl GuiWispControlWidget {
    pub fn new() -> Self {
        Self {
            last_form_change: core::cell::Cell::new(-FORM_COOLDOWN - 1),
            ..Default::default()
        }
    }

    fn movement(&self) -> (i32, i32) {
        (
            i32::from(self.right.get()) - i32::from(self.left.get()),
            i32::from(self.down.get()) - i32::from(self.up.get()),
        )
    }

    fn update_movement(&self, key: i32, pressed: bool) -> EventResult {
        match key {
            KEY_A | KEY_LEFT => self.left.set(pressed),
            KEY_D | KEY_RIGHT => self.right.set(pressed),
            KEY_W | KEY_UP => self.up.set(pressed),
            KEY_S | KEY_DOWN => self.down.set(pressed),
            _ => return EventResult::Ignored,
        }
        let (x, z) = self.movement();
        // Key repeat sends presses for keys already down: tell the rules only
        // about changes.
        if self.sent_movement.replace((x, z)) != (x, z) {
            send(Command::Move { x, z });
        }
        EventResult::Handled
    }
}

impl Widget<Global> for GuiWispControlWidget {
    fn name(&self) -> &'static str {
        "GuiWispControl"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        // The WASM archive does not load the Lua keybind widget. Install the
        // same game-local binding at the engine level as a fallback for input
        // paths that do not deliver Ctrl-Q to the WASM receiver.
        let _ = spring::send_commands("unbind", "ctrl+q");
        let _ = spring::send_commands("bind", "ctrl+q reloadforce");
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        // The player never selects anything: the wisp is steered directly.
        if spring::selection::get_selected_units_count(0).unwrap_or(0) > 0 {
            let _ = spring::selection::select_unit_array(&[], false);
        }
    }

    fn draw_screen(&self, _ctx: &spring_addons::AddonContext<'_, Global>, width: i32, height: i32) {
        self.screen_width.set(width);
        self.screen_height.set(height);
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        if event.matches("r") && ctx.global().player.access().game_over_sequence {
            // The Lua endgame's Restart button reloads the current game.
            // Use the Core lifecycle API so this resets the WASM module
            // and synced state without leaving the WASM menu.
            crate::restart::restart();
            return EventResult::Handled;
        }
        if event.ctrl && event.matches("q") {
            // Return to the WASM menu in the current engine process.  Calling
            // system_control::quit exits the process, which skips the menu.
            let _ = spring::send_commands("reloadforce", "");
            return EventResult::Handled;
        }
        if event.matches("space") {
            send(Command::SwitchForm);
            let mut player = ctx.global().player.access_mut();
            // Only dismiss the hint once it is actually on screen.  Latching on
            // any space press meant tapping space before collecting the eyes
            // suppressed the prompt for the rest of the game.
            let has_eyes = player.parts.contains(BodyPart::Eyes);
            if player.spirit && has_eyes {
                player.space_used = true;
            }
            // Predict the same result the rules module will compute, so the UI
            // and the synced side cannot disagree about what was asked for:
            // leaving spirit form needs eyes, otherwise it stays spirit.
            let current = player.spirit;
            let wanted = !(current && has_eyes);
            // Mirror the rules module's form-change cooldown.  Predicting a
            // switch it is going to refuse just means the prediction snaps back
            // when it times out.
            let now = crate::state::current_frame();
            let off_cooldown = now - self.last_form_change.get() > FORM_COOLDOWN;
            if wanted != current && off_cooldown {
                self.last_form_change.set(now);
                player.form.begin(current, wanted);
                player.spirit = wanted;
                let wisp = player.wisp;
                player.pending_spirit = Some(wanted);
                player.pending_since = crate::state::current_frame();
                let position = spring::get_unit_position(player.wisp, false, false)
                    .unwrap_or_else(|_| spring::types::Float3::default());
                drop(player);
                // unit_wisp_controller.lua plays this when the form changes.  It
                // has to be here, not in api_sync: the prediction above has
                // already flipped our copy of `spirit`, so by the time the
                // snapshot arrives there is no change left for it to notice.
                let _ = spring::sound::play_sound_file(
                    "sounds/changestate.ogg",
                    1.0,
                    position,
                    spring::types::Float3::default(),
                    0,
                );
                // Show the body now.  Waiting for the rules module to answer
                // and api_sync to apply no-draw costs a visible round trip.
                if wisp >= 0 {
                    let _ = spring::unsynced_ctrl::set_unit_no_draw(wisp, wanted);
                }
            }
            return EventResult::Handled;
        }
        self.update_movement(event.key_code, true)
    }

    fn key_release(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        self.update_movement(event.key_code, false)
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let (center_x, center_y) = (self.screen_width.get() / 2, self.screen_height.get() / 2);
        let in_restart_prompt = ctx.global().player.access().game_over_sequence
            && button == 1
            && (x - center_x).abs() <= RESTART_PROMPT_HALF_WIDTH
            && (center_y + RESTART_PROMPT_TOP..=center_y + RESTART_PROMPT_BOTTOM).contains(&y);
        if in_restart_prompt {
            // Match the Lua endgame Restart button as well as the R shortcut.
            crate::restart::restart();
            return EventResult::Handled;
        }
        let global = ctx.global().player.access();
        // Match ui_wisp_control.lua: only the embodied, armed player can pull a
        // nearby lever.  In particular, tracing a lever is not enough while the
        // player is a spirit or looking across the map.
        if button != 1
            || global.game_over
            || global.spirit
            || !global.parts.contains(BodyPart::Arms)
        {
            // Claim the middle button as well. The engine only gives its
            // camera receiver the drag when no input receiver handles the
            // press; handling it here disables camera rotation.
            return (button == 1 || button == 2 || button == 3).into();
        }
        // Prefer the lever the highlight widget is already glowing: it traced
        // the cursor this frame, and a second TraceScreenRay in the same frame
        // interferes with the first and comes back with no hit at all.  Falling
        // back to our own trace keeps clicks working if the glow never
        // initialised (no GL, or a shader that failed to build).
        let hovered = *ctx.global().hovered_lever.access();
        let unit_id = if let Some(hovered) = hovered {
            hovered
        } else {
            let Ok(trace) = spring::camera::trace_screen_ray(
                x as f32,
                y as f32,
                spring::camera::TraceScreenRayOptions {
                    only_coords: false,
                    use_minimap: false,
                    include_sky: false,
                    ignore_water: false,
                    height_offset: 0.0,
                },
            ) else {
                return EventResult::Handled;
            };
            trace.hit_id
        };
        if unit_def(unit_id) != defs().lever {
            return EventResult::Handled;
        }
        let wisp = global.wisp;
        let (Ok(lever_position), Ok(wisp_position)) = (
            spring::get_unit_position(unit_id, false, false),
            spring::get_unit_position(wisp, false, false),
        ) else {
            return EventResult::Handled;
        };
        let dx = lever_position.x - wisp_position.x;
        let dz = lever_position.z - wisp_position.z;
        if dx * dx + dz * dz >= 100.0 * 100.0 {
            return EventResult::Handled;
        }
        send(Command::PullLever(unit_id));
        EventResult::Handled
    }
}
