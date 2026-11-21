//! Modal UI overlays: Pause, Settings, LevelUp selection, and GameOver summary.

use super::world_render::render_world;
use super::{draw_centered_text, ui_button_v, CYAN, GOLD, PINK};
use crate::game::{Game, ScreenState, Viewport};
use macroquad::prelude::*;

pub fn render_paused(game: &mut Game, vp: Viewport) {
    render_world(game, vp);
    let s = vp.scale;
    let vw = vp.vw;
    let vh = vp.vh;

    draw_rectangle(0.0, 0.0, vp.sw, vp.sh, Color::from_rgba(4, 6, 11, 210));
    draw_centered_text("PAUZA", vw * 0.5 * s, vh * 0.25 * s, 38.0 * s, CYAN);

    let bw = 270.0;
    let bx = vw * 0.5 - bw * 0.5;
    let by = vh * 0.36;
    if ui_button_v(Rect::new(bx, by, bw, 52.0), s, "WZNOW", None, CYAN)
        || is_key_pressed(KeyCode::Escape)
    {
        game.resume_run();
    }
    if ui_button_v(Rect::new(bx, by + 66.0, bw, 52.0), s, "OPCJE", None, GOLD) {
        game.prev_state = ScreenState::Paused;
        game.state = ScreenState::Settings;
    }
    if ui_button_v(
        Rect::new(bx, by + 132.0, bw, 52.0),
        s,
        "PRZERWIJ LOT",
        None,
        PINK,
    ) {
        game.finish_run(false);
    }
}

pub fn render_settings(game: &mut Game, vp: Viewport) {
    let s = vp.scale;
    let vw = vp.vw;
    draw_centered_text("OPCJE SYSTEMU", vw * 0.5 * s, 56.0 * s, 32.0 * s, CYAN);
    let bw = (vw - vp.safe_x * 2.0).min(380.0);
    let bx = vw * 0.5 - bw * 0.5;

    let s_lbl = if game.settings.sound {
        "DZWIEK: WLACZONY"
    } else {
        "DZWIEK: WYLACZONY"
    };
    if ui_button_v(Rect::new(bx, 92.0, bw, 48.0), s, s_lbl, None, CYAN) {
        game.settings.sound = !game.settings.sound;
    }
    let sh_lbl = if game.settings.shake {
        "SHAKE: WLACZONY"
    } else {
        "SHAKE: WYLACZONY"
    };
    if ui_button_v(Rect::new(bx, 150.0, bw, 48.0), s, sh_lbl, None, CYAN) {
        game.settings.shake = !game.settings.shake;
    }
    let dm_lbl = if game.settings.dmg_nums {
        "OBRAZENIA: WLACZONE"
    } else {
        "OBRAZENIA: WYLACZONE"
    };
    if ui_button_v(Rect::new(bx, 208.0, bw, 48.0), s, dm_lbl, None, CYAN) {
        game.settings.dmg_nums = !game.settings.dmg_nums;
    }
    let st_lbl = if game.settings.fixed_stick {
        "DRAZEK: STALY"
    } else {
        "DRAZEK: PLYWAJACY"
    };
    if ui_button_v(Rect::new(bx, 266.0, bw, 48.0), s, st_lbl, None, CYAN) {
        game.settings.fixed_stick = !game.settings.fixed_stick;
    }
    let ai_lbl = if game.settings.auto_aim {
        "AUTO-AIM: WLACZONY"
    } else {
        "AUTO-AIM: WYLACZONY"
    };
    if ui_button_v(Rect::new(bx, 324.0, bw, 48.0), s, ai_lbl, None, CYAN) {
        game.settings.auto_aim = !game.settings.auto_aim;
    }
    if ui_button_v(Rect::new(bx, 392.0, bw, 52.0), s, "WROC", None, GOLD)
        || is_key_pressed(KeyCode::Escape)
    {
        game.touch.clear();
        game.state = game.prev_state;
    }
}

pub fn render_level_up(game: &mut Game, vp: Viewport) {
    render_world(game, vp);
    let s = vp.scale;
    let vw = vp.vw;

    draw_rectangle(0.0, 0.0, vp.sw, vp.sh, Color::from_rgba(4, 6, 11, 215));
    draw_centered_text("OVERCLOCK RDZENIA", vw * 0.5 * s, 68.0 * s, 32.0 * s, CYAN);
    let cw = (vw - vp.safe_x * 2.0).min(420.0);
    let choices = game.level_choices;
    for (i, opt) in choices.iter().enumerate() {
        let rect = Rect::new(vw * 0.5 - cw * 0.5, 110.0 + i as f32 * 86.0, cw, 72.0);
        if ui_button_v(rect, s, opt.title, Some(opt.desc), CYAN) {
            game.apply_upgrade(opt.id);
            break;
        }
    }
}

pub fn render_game_over(game: &mut Game, vp: Viewport, victory: bool) {
    let s = vp.scale;
    let vw = vp.vw;
    let vh = vp.vh;

    let title = if victory {
        "SEKTOR DEMO OCZYSZCZONY"
    } else {
        "RDZEN ZNISZCZONY"
    };
    let diff = game.difficulty();
    let ship = game.selected_ship();

    draw_centered_text(
        title,
        vw * 0.5 * s,
        vh * 0.22 * s,
        34.0 * s,
        if victory { CYAN } else { PINK },
    );
    draw_centered_text(
        &format!("{}  |  {} ({:.1}x)", ship.name, diff.name, diff.shard_mult),
        vw * 0.5 * s,
        (vh * 0.22 + 34.0) * s,
        17.0 * s,
        diff.color(),
    );
    if game.is_new_record {
        draw_centered_text(
            "* NOWY REKORD OSOBISTY! *",
            vw * 0.5 * s,
            (vh * 0.22 + 62.0) * s,
            19.0 * s,
            GOLD,
        );
    }

    let stat_y = vh * 0.22 + if game.is_new_record { 96.0 } else { 76.0 };
    draw_centered_text(
        &format!("FALA: {}  |  POZIOM: {}", game.wave, game.level),
        vw * 0.5 * s,
        stat_y * s,
        22.0 * s,
        WHITE,
    );
    draw_centered_text(
        &format!(
            "ZNISZCZENI: {}  |  SHARDS: +{}",
            game.kills, game.run_shards
        ),
        vw * 0.5 * s,
        (stat_y + 32.0) * s,
        22.0 * s,
        GOLD,
    );
    draw_centered_text(
        &format!(
            "NAJLEPSZA FALA: {}  |  REKORD KILLS: {}",
            game.meta.best_wave, game.meta.best_kills
        ),
        vw * 0.5 * s,
        (stat_y + 62.0) * s,
        16.0 * s,
        LIGHTGRAY,
    );
    if ui_button_v(
        Rect::new(vw * 0.5 - 150.0, stat_y + 92.0, 300.0, 54.0),
        s,
        "POWROT DO HANGARU",
        None,
        CYAN,
    ) {
        game.touch.clear();
        game.state = ScreenState::Menu;
    }
}
