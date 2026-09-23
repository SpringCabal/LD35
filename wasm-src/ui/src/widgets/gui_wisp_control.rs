use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::state::Global;

use partsneeded_shared::FORM_COOLDOWN;

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

// Match camera_control.lua. The target is smoothed in the widget; the engine
// transition itself stays at zero so the two interpolation layers do not fight
// each other.
const CAMERA_FOLLOW_TAU: f32 = 0.12;

// SpringApp::Reload treats an empty script as "return to menu". The WASM menu
// normally supplies this script when launching Parts Needed, so preserve that
// same launch context for the in-game Restart action.
const RESTART_SCRIPT: &str = r#"[GAME]
{
    MapName=LD35 map 0.1;
    GameMode=0;
    GameType=Parts Needed WASM 0.1.0-wasm;

    NumTeams=2;
    NumUsers=2;

    HostIP=127.0.0.1;
    HostPort=8452;
    IsHost=1;
    NumPlayers=1;
    GameStartDelay=0;

    StartMetal=1000;
    StartEnergy=1000;

    StartposType=3;
    LimitDGun=0;
    DiminishingMMs=0;
    GhostedBuildings=1;
    MyPlayerNum=1;
    MyPlayerName=0;
    NumRestrictions=0;
    MaxSpeed=20;
    MinSpeed=0.1;

    [TEAM0]
    {
        AllyTeam=0;
        Side=;
        RGBColor=0.35294118523598 0.35294118523598 1;
        TeamLeader=1;
        Handicap=0;
        StartPosX=0;
        StartPosZ=0;
    }
    [PLAYER1]
    {
        Name=0;
        Spectator=0;
        Team=0;
    }
    [TEAM1]
    {
        AllyTeam=1;
        Side=;
        RGBColor=0.7843137383461 0 0;
        TeamLeader=1;
        Handicap=0;
        StartPosX=0;
        StartPosZ=0;
    }
    [AI1]
    {
        Name=1: NullAI;
        ShortName=NullAI;
        Team=1;
        IsFromDemo=0;
        Host=1;
        [Options] {}
    }
    [ALLYTEAM0]
    {
        NumAllies=0;
    }
    [ALLYTEAM1]
    {
        NumAllies=0;
    }
    [MODOPTIONS]
    {
        gamemode=play;
    }
}
"#;

fn restart_current_game() {
    let _ = spring::reload(RESTART_SCRIPT);
}

#[derive(Clone, Copy, Default)]
struct CameraTracking {
    wisp_id: i32,
    focus: spring::Float3,
    initialized: bool,
}

/// Direct player control, ported from `ui_wisp_control.lua`, plus the tracking
/// camera from `camera_control.lua`.
#[derive(Default)]
pub struct GuiWispControlWidget {
    left: core::cell::Cell<bool>,
    right: core::cell::Cell<bool>,
    up: core::cell::Cell<bool>,
    down: core::cell::Cell<bool>,
    scan_ticks: core::cell::Cell<i32>,
    last_form_change: core::cell::Cell<i32>,
    screen_width: core::cell::Cell<i32>,
    screen_height: core::cell::Cell<i32>,
    camera_tracking: core::cell::RefCell<CameraTracking>,
}

impl GuiWispControlWidget {
    pub fn new() -> Self {
        Self {
            last_form_change: core::cell::Cell::new(-FORM_COOLDOWN - 1),
            camera_tracking: core::cell::RefCell::new(CameraTracking {
                wisp_id: -1,
                ..Default::default()
            }),
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
        let _ = spring::send_lua_rules_msg(&alloc::format!("movement|{x}|{z}"));
        EventResult::Handled
    }

    fn lock_camera(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let wisp = ctx.global().player.access().wisp;
        if wisp < 0 {
            return;
        }
        // camera_control.lua follows the unit's view position: the raw physics
        // position moves in simulation steps and would make the camera jitter.
        let Ok(position) = spring::unit_rendering::get_unit_view_position(wisp, false) else {
            return;
        };
        let target = position;

        let focus = {
            let mut tracking = self.camera_tracking.borrow_mut();
            // Clamp the first sample (and every newly discovered UnitCreated
            // equivalent) directly to the live unit. Interpolating from the
            // zero/default point would cause a visible camera sweep at start.
            if !tracking.initialized || tracking.wisp_id != wisp {
                tracking.wisp_id = wisp;
                tracking.focus = target;
                tracking.initialized = true;
            } else if dt.is_finite() && dt > 0.0 {
                let alpha = 1.0 - libm::expf(-(dt / CAMERA_FOLLOW_TAU));
                tracking.focus.x += (target.x - tracking.focus.x) * alpha;
                tracking.focus.y += (target.y - tracking.focus.y) * alpha;
                tracking.focus.z += (target.z - tracking.focus.z) * alpha;
            }
            tracking.focus
        };

        // Lua uses SetCameraTarget(..., 0): the smoothing above is the only
        // interpolation. Supplying no transition time would add the engine's
        // default 0.5-second transition on top and make the camera laggy.
        let _ = spring::camera::set_camera_target(
            focus,
            spring::camera::SetCameraTargetOptions {
                transition_time: Some(0.0),
                dir_x: None,
                dir_y: None,
                dir_z: None,
            },
        );
    }

    fn refresh_npc_wisps(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        // During the ending new ceremonial wisps must enter the billboard list
        // immediately. The normal 30-frame scan interval is useful in the
        // dungeon, but made each arrival briefly show its physical model.
        let ending = ctx.global().player.access().game_over;
        if !ending && self.scan_ticks.get() > 0 {
            self.scan_ticks.set(self.scan_ticks.get() - 1);
            return;
        }
        self.scan_ticks.set(if ending { 0 } else { 30 });
        let (Ok(wisp_def), Ok(npc_def)) = (
            spring::get_unit_def_id_by_name("wisp"),
            spring::get_unit_def_id_by_name("npcwisp"),
        ) else {
            return;
        };
        let Ok(all_units) = spring::get_all_units() else {
            return;
        };
        if let Some(player) = all_units
            .iter()
            .copied()
            .find(|unit_id| spring::units_info::get_unit_def_id(*unit_id).ok() == Some(wisp_def))
        {
            let spirit = {
                let mut state = ctx.global().player.access_mut();
                state.wisp = player;
                state.spirit
            };
            if spirit {
                let _ = spring::unsynced_ctrl::set_unit_no_draw(player, true);
            }
        }
        // unit_wisp_draw.lua hides every light unit: they exist to place a
        // deferred light, their model is never meant to be seen.  Leaving them
        // drawn scattered visible props through the level.
        if let Ok(light_def) = spring::get_unit_def_id_by_name("light") {
            for unit_id in all_units
                .iter()
                .filter(|id| spring::units_info::get_unit_def_id(**id).ok() == Some(light_def))
            {
                let _ = spring::unsynced_ctrl::set_unit_no_draw(*unit_id, true);
            }
        }
        let wisps: Vec<i32> = all_units
            .into_iter()
            .filter(|unit_id| spring::units_info::get_unit_def_id(*unit_id).ok() == Some(npc_def))
            .collect();
        for unit_id in &wisps {
            let _ = spring::unsynced_ctrl::set_unit_no_draw(*unit_id, true);
        }
        ctx.global().npcs.access_mut().wisps = wisps;
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

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let (x, z) = self.movement();
        if x != 0 || z != 0 {
            let _ = spring::send_lua_rules_msg(&alloc::format!("movement|{x}|{z}"));
        }
        self.refresh_npc_wisps(ctx);
        self.lock_camera(ctx, dt);
        let _ = spring::selection::select_unit_array(&[], false);
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
            restart_current_game();
            return EventResult::Handled;
        }
        if event.ctrl && event.matches("q") {
            // Return to the WASM menu in the current engine process.  Calling
            // system_control::quit exits the process, which skips the menu.
            let _ = spring::send_commands("reloadforce", "");
            return EventResult::Handled;
        }
        if event.matches("space") {
            let _ = spring::send_lua_rules_msg("switch_form");
            let mut player = ctx.global().player.access_mut();
            // Only dismiss the hint once it is actually on screen.  Latching on
            // any space press meant tapping space before collecting the eyes
            // suppressed the prompt for the rest of the game.
            if player.spirit && player.eyes {
                player.space_used = true;
            }
            // Predict the same result the rules module will compute, so the UI
            // and the synced side cannot disagree about what was asked for:
            // leaving spirit form needs eyes, otherwise it stays spirit.
            let current = player.spirit;
            let wanted = !(current && player.eyes);
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
            restart_current_game();
            return EventResult::Handled;
        }
        let global = ctx.global().player.access();
        // Match ui_wisp_control.lua: only the embodied, armed player can pull a
        // nearby lever.  In particular, tracing a lever is not enough while the
        // player is a spirit or looking across the map.
        if button != 1 || global.game_over || global.spirit || global.body_mask & 4 == 0 {
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
        let hovered = crate::widgets::gfx_highlight::hovered_lever();
        let unit_id = if hovered >= 0 {
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
        let Ok(lever_def) = spring::get_unit_def_id_by_name("lever") else {
            return EventResult::Handled;
        };
        if spring::get_unit_def_id(unit_id)
            .ok()
            .is_none_or(|def| def.0 != lever_def)
        {
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
        let _ = spring::send_lua_rules_msg(&alloc::format!("pull_lever|{unit_id}"));
        EventResult::Handled
    }

    fn mouse_wheel(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _up: bool,
        _value: f32,
    ) -> EventResult {
        // The authored camera is fixed. Claim the wheel at the input receiver
        // boundary so the engine never applies its default zoom operation.
        EventResult::Handled
    }
}
