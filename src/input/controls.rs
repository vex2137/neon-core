//! Unified per-frame simulation input snapshot (keyboard + mouse + touch + gamepad).

use super::gamepad::GamepadSnapshot;
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
    pub fire_held: bool,
    pub dash_pressed: bool,
    pub pause_pressed: bool,
}

/// Pure input merger combining touch, keyboard, mouse, and gamepad inputs into `FrameInput`.
#[allow(clippy::too_many_arguments)]
pub fn merge_playing_inputs(
    touch_move: Vec2,
    kb_move: Vec2,
    gamepad: GamepadSnapshot,
    touch_aim: Option<Vec2>,
    mouse_aim: Option<Vec2>,
    mouse_fire: bool,
    dash_triggered: bool,
    pause_triggered: bool,
) -> FrameInput {
    if pause_triggered || gamepad.pause_pressed {
        return FrameInput {
            pause_pressed: true,
            ..FrameInput::default()
        };
    }

    let mut mv = touch_move + kb_move + gamepad.move_dir;
    if mv.length_squared() > 1.0 {
        mv = mv.normalize();
    }

    // Precedence: Gamepad Right Stick (twin-stick overrides auto-aim) -> Touch Right Stick -> Mouse.
    let manual_aim_dir = gamepad.aim_dir.or(touch_aim).or(mouse_aim);
    let fire_held = mouse_fire || touch_aim.is_some() || gamepad.fire_held;

    FrameInput {
        move_dir: mv,
        manual_aim_dir,
        fire_held,
        dash_pressed: dash_triggered || gamepad.dash_pressed,
        pause_pressed: false,
    }
}

/// Polls Macroquad keyboard, mouse, and touch inputs and merges them with the active gamepad snapshot.
pub fn poll_playing_input(
    touch_state: &mut TouchState,
    gamepad: GamepadSnapshot,
    vp: Viewport,
    settings: Settings,
) -> FrameInput {
    let touch_list = touches();
    let mut dash_triggered = false;
    let mut pause_triggered = false;

    // Desktop mouse clicks on Pause / Dash HUD buttons when no touch is active.
    if touch_list.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let vm = vec2(mx / vp.scale, my / vp.scale);
        if pause_button_rect(vp).contains(vm) {
            touch_state.clear();
            return FrameInput {
                pause_pressed: true,
                ..FrameInput::default()
            };
        }
        if vm.distance_squared(dash_button_center(vp)) <= 52.0 * 52.0 {
            dash_triggered = true;
        }
    }

    let touch_actions = touch_state.process_touches(&touch_list, vp, settings);
    if touch_actions.pause_pressed {
        return FrameInput {
            pause_pressed: true,
            ..FrameInput::default()
        };
    }
    if touch_actions.dash_pressed {
        dash_triggered = true;
    }

    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
        touch_state.clear();
        pause_triggered = true;
    }
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift) {
        dash_triggered = true;
    }

    let mut kb_move = Vec2::ZERO;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        kb_move.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        kb_move.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        kb_move.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        kb_move.x += 1.0;
    }

    let touch_aim = if touch_state.right_stick.touch_id.is_some()
        && touch_state.right_stick.dir.length_squared() > 0.01
    {
        Some(touch_state.right_stick.dir)
    } else {
        None
    };

    let mouse_fire = touch_list.is_empty() && is_mouse_button_down(MouseButton::Left);
    let mouse_aim = if touch_list.is_empty() && !settings.auto_aim && mouse_fire {
        let (mx, my) = mouse_position();
        let center = vec2(vp.sw * 0.5, vp.sh * 0.5);
        let delta = vec2(mx, my) - center;
        if delta.length_squared() > 16.0 {
            Some(delta.normalize())
        } else {
            None
        }
    } else {
        None
    };

    merge_playing_inputs(
        touch_state.left_stick.dir,
        kb_move,
        gamepad,
        touch_aim,
        mouse_aim,
        mouse_fire,
        dash_triggered,
        pause_triggered,
    )
}
