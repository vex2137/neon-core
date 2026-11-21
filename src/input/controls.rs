//! Unified per-frame simulation input snapshot (keyboard + mouse + touch).

use super::touch::{dash_button_center, pause_button_rect, TouchState};
use crate::game::config::Viewport;
use crate::game::state::Settings;
use macroquad::math::{vec2, Vec2};
use macroquad::prelude::{
    is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position,
    touches, KeyCode, MouseButton,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    pub move_dir: Vec2,
    pub manual_aim_dir: Option<Vec2>,
    pub dash_pressed: bool,
    pub pause_pressed: bool,
}

/// Polls Macroquad keyboard, mouse, and touch inputs into a clean `FrameInput`.
pub fn poll_playing_input(
    touch_state: &mut TouchState,
    vp: Viewport,
    settings: Settings,
) -> FrameInput {
    let mut input = FrameInput::default();
    let touch_list = touches();

    // Desktop mouse clicks on Pause / Dash HUD buttons when no touch is active.
    if touch_list.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let vm = vec2(mx / vp.scale, my / vp.scale);
        if pause_button_rect(vp).contains(vm) {
            touch_state.clear();
            input.pause_pressed = true;
            return input;
        }
        if vm.distance_squared(dash_button_center(vp)) <= 52.0 * 52.0 {
            input.dash_pressed = true;
        }
    }

    let touch_actions = touch_state.process_touches(&touch_list, vp, settings);
    if touch_actions.pause_pressed {
        input.pause_pressed = true;
        return input;
    }
    if touch_actions.dash_pressed {
        input.dash_pressed = true;
    }

    if is_key_pressed(KeyCode::Escape) {
        touch_state.clear();
        input.pause_pressed = true;
        return input;
    }
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift) {
        input.dash_pressed = true;
    }

    // Combine touch movement stick and keyboard WASD / Arrow keys.
    let mut mv = touch_state.left_stick.dir;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        mv.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        mv.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        mv.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        mv.x += 1.0;
    }
    if mv.length_squared() > 1.0 {
        mv = mv.normalize();
    }
    input.move_dir = mv;

    // Manual aim: touch right stick takes precedence, followed by desktop mouse cursor when auto-aim is off.
    if touch_state.right_stick.touch_id.is_some()
        && touch_state.right_stick.dir.length_squared() > 0.01
    {
        input.manual_aim_dir = Some(touch_state.right_stick.dir);
    } else if touch_list.is_empty() && !settings.auto_aim && is_mouse_button_down(MouseButton::Left)
    {
        let (mx, my) = mouse_position();
        let center = vec2(vp.sw * 0.5, vp.sh * 0.5);
        let delta = vec2(mx, my) - center;
        if delta.length_squared() > 16.0 {
            input.manual_aim_dir = Some(delta.normalize());
        }
    }

    input
}
