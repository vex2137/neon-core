//! NEON CORE — Native Rust (Macroquad 0.4) arcade twin-stick survival shooter.
//!
//! Entry point, window configuration, Android JNI focus-loss hook, gamepad polling, and main loop dispatch.

mod audio;
mod combat;
mod effects;
mod enemies;
mod game;
mod input;
mod performance;
mod persistence;
mod player;
mod progression;
mod rng;
mod ui;
mod world;

#[cfg(test)]
mod tests;

use audio::SoundBank;
use game::{Game, ScreenState, Viewport};
use input::{poll_playing_input, MenuNavInput};
use macroquad::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

static ANDROID_PAUSE_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_dev_vex_neon_1core_MainActivity_neonOnPause(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) {
    ANDROID_PAUSE_REQUESTED.store(true, Ordering::SeqCst);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "NEON CORE".to_owned(),
        high_dpi: true,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(SoundBank::empty());
    let mut audio_initialized = false;

    loop {
        let sw = screen_width();
        let sh = screen_height();
        let focus_lost = ANDROID_PAUSE_REQUESTED.swap(false, Ordering::SeqCst);

        // Guard against collapsed/minimized surfaces on Android & desktop window transitions.
        if !sw.is_finite() || !sh.is_finite() || sw < 16.0 || sh < 16.0 {
            game.pause_run();
            next_frame().await;
            continue;
        }

        if !audio_initialized {
            audio_initialized = true;
            game.sounds.load_all().await;
        }

        let raw_dt = get_frame_time();
        if focus_lost || (raw_dt.is_finite() && raw_dt > 0.35) {
            if game.state == ScreenState::Playing {
                game.pause_run();
            } else if focus_lost && game.state == ScreenState::LevelUp {
                game.save_active_run();
            }
        }

        let dt = if raw_dt.is_finite() {
            raw_dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        clear_background(Color::from_rgba(4, 6, 11, 255));

        let vp = Viewport::current();
        let gp_snap = game.gamepad.poll(dt, game.settings);

        // Track active input modality so UI focus highlight appears for gamepad/keyboard nav
        // and hides cleanly when tapping or clicking with mouse/touch.
        if gp_snap.any_activity {
            game.ui_focus.gamepad_nav_active = true;
        } else if is_mouse_button_pressed(MouseButton::Left)
            || touches().iter().any(|t| t.phase == TouchPhase::Started)
        {
            game.ui_focus.gamepad_nav_active = false;
        }

        match game.state {
            ScreenState::Menu => {
                let nav = MenuNavInput::from_gamepad_and_keyboard(&gp_snap);
                ui::menus::render_menu(&mut game, vp, nav);
            }
            ScreenState::Playing => {
                if dt > 0.0 {
                    let frame_input =
                        poll_playing_input(&mut game.touch, gp_snap, vp, game.settings);
                    game.step_playing(dt, frame_input, vp);
                }
                ui::world_render::render_world(&game, vp);
                ui::hud::render_hud(&game, vp);
            }
            ScreenState::Paused => {
                let nav = MenuNavInput::from_gamepad_and_keyboard(&gp_snap);
                ui::overlays::render_paused(&mut game, vp, nav, gp_snap.pause_pressed);
            }
            ScreenState::Settings => {
                let nav = MenuNavInput::from_gamepad_and_keyboard(&gp_snap);
                ui::overlays::render_settings(&mut game, vp, nav);
            }
            ScreenState::LevelUp => {
                let nav = MenuNavInput::from_gamepad_and_keyboard(&gp_snap);
                ui::overlays::render_level_up(&mut game, vp, nav);
            }
            ScreenState::GameOver { victory } => {
                let nav = MenuNavInput::from_gamepad_and_keyboard(&gp_snap);
                ui::overlays::render_game_over(&mut game, vp, victory, nav);
            }
        }

        next_frame().await;
    }
}
