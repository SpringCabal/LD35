use spring::mouse;
use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::commands::send;
use crate::defs::{defs, unit_def};
use crate::state::Global;

use partsneeded_shared::{BodyPart, Command, FORM_COOLDOWN, LEVER_REACH};
use springcabal::menu;
use springcabal_control::keyboard::{KeyMovement, send_direction};

const RESTART_PROMPT_HALF_WIDTH: i32 = 190;
const RESTART_PROMPT_TOP: i32 = -52;
const RESTART_PROMPT_BOTTOM: i32 = -12;

#[derive(Default)]
pub struct GuiWispControlWidget {
    movement: KeyMovement,
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

    fn update_movement(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        event: &KeyEvent<'_>,
        pressed: bool,
    ) -> EventResult {
        let claimed = self.movement.key(spring, event, pressed);
        if claimed {
            self.movement
                .send(spring, |direction| send_direction(spring, direction));
        }
        claimed.into()
    }
}

impl Widget<Global> for GuiWispControlWidget {
    fn name(&self) -> &'static str {
        "GuiWispControl"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = _ctx.spring();
        self.movement
            .send(spring, |direction| send_direction(spring, direction));
        if spring.selection().get_selected_units_count().unwrap_or(0) > 0 {
            let _ = spring.selection().select_unit_array(&[], false);
        }
    }

    fn draw_screen(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        self.screen_width.set(width);
        self.screen_height.set(height);
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        let spring = ctx.spring();
        if event.matches("r") && ctx.global().player.access().end_screen_shown(spring) {
            send(spring, Command::Restart);
            return EventResult::Handled;
        }
        if menu::is_quit_to_menu(event) {
            menu::quit_to_menu(spring);
            return EventResult::Handled;
        }
        if event.matches("space") {
            send(spring, Command::SwitchForm);
            let mut player = ctx.global().player.access_mut();
            let has_eyes = player.parts.contains(BodyPart::Eyes);
            if player.spirit && has_eyes {
                player.space_used = true;
            }
            let current = player.spirit;
            let wanted = !(current && has_eyes);
            let now = spring.game().current_frame().unwrap_or(0);
            let off_cooldown = now - self.last_form_change.get() > FORM_COOLDOWN;
            if wanted != current && off_cooldown {
                self.last_form_change.set(now);
                player.form.begin(spring, current, wanted);
                player.spirit = wanted;
                let wisp = player.wisp;
                player.pending_spirit = Some(wanted);
                player.pending_since = spring.game().current_frame().unwrap_or(0);
                drop(player);
                let position = spring
                    .units_info()
                    .get_unit_position(wisp, Default::default())
                    .unwrap_or_else(|_| spring::Float3::default());
                let _ = spring.sound().play_sound_file(
                    "sounds/changestate.ogg",
                    1.0,
                    position,
                    spring::Float3::default(),
                    0,
                );
                crate::show_body(spring, wisp, !wanted);
            }
            return EventResult::Handled;
        }
        self.update_movement(spring, event, true)
    }

    fn key_release(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        let spring = _ctx.spring();
        self.update_movement(spring, event, false)
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let spring = ctx.spring();
        let (center_x, center_y) = (self.screen_width.get() / 2, self.screen_height.get() / 2);
        let in_restart_prompt = ctx.global().player.access().end_screen_shown(spring)
            && button == mouse::LEFT
            && (x - center_x).abs() <= RESTART_PROMPT_HALF_WIDTH
            && (center_y + RESTART_PROMPT_TOP..=center_y + RESTART_PROMPT_BOTTOM).contains(&y);
        if in_restart_prompt {
            send(spring, Command::Restart);
            return EventResult::Handled;
        }
        let global = ctx.global().player.access();
        if button != mouse::LEFT
            || global.game_over
            || global.spirit
            || !global.parts.contains(BodyPart::Arms)
        {
            return matches!(button, mouse::LEFT | mouse::MIDDLE | mouse::RIGHT).into();
        }
        let wisp = global.wisp;
        drop(global);
        let hovered = *ctx.global().hovered_lever.access();
        let unit_id = if let Some(hovered) = hovered {
            hovered
        } else {
            let Some(unit_id) = spring.camera().unit_at(x as f32, y as f32).ok().flatten() else {
                return EventResult::Handled;
            };
            unit_id
        };
        if unit_def(spring, unit_id) != defs(spring).lever {
            return EventResult::Handled;
        }
        let (Ok(lever_position), Ok(wisp_position)) = (
            spring
                .units_info()
                .get_unit_position(unit_id, Default::default()),
            spring
                .units_info()
                .get_unit_position(wisp, Default::default()),
        ) else {
            return EventResult::Handled;
        };
        let dx = lever_position.x - wisp_position.x;
        let dz = lever_position.z - wisp_position.z;
        if dx * dx + dz * dz >= LEVER_REACH * LEVER_REACH {
            return EventResult::Handled;
        }
        send(spring, Command::PullLever(unit_id));
        EventResult::Handled
    }
}
