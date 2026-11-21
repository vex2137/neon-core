//! NEON CORE — Native Rust (Macroquad 0.4) arcade twin-stick survival shooter.
//!
//! Entry point, window configuration, Android JNI focus-loss hook, and main loop dispatch.

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
use input::poll_playing_input;
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
        if (focus_lost || (raw_dt.is_finite() && raw_dt > 0.35))
            && game.state == ScreenState::Playing
        {
            game.pause_run();
        }

        let dt = if raw_dt.is_finite() {
            raw_dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        clear_background(Color::from_rgba(4, 6, 11, 255));

        let vp = Viewport::current();

        match game.state {
            ScreenState::Menu => ui::menus::render_menu(&mut game, vp),
            ScreenState::Playing => {
                if dt > 0.0 {
                    let frame_input = poll_playing_input(&mut game.touch, vp, game.settings);
                    game.step_playing(dt, frame_input, vp);
                }
                ui::world_render::render_world(&game, vp);
                ui::hud::render_hud(&game, vp);
            }
            ScreenState::Paused => ui::overlays::render_paused(&mut game, vp),
            ScreenState::Settings => ui::overlays::render_settings(&mut game, vp),
            ScreenState::LevelUp => ui::overlays::render_level_up(&mut game, vp),
            ScreenState::GameOver { victory } => {
                ui::overlays::render_game_over(&mut game, vp, victory);
            }
        }

        next_frame().await;
    }
}
