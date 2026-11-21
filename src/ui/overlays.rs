//! Modal UI overlays: Pause (with Save & Quit), expanded two-column Settings, LevelUp draft, and GameOver.

use super::world_render::render_world;
use super::{
    draw_centered_text, ui_button_hit_x, ui_button_v_focused, CYAN, GOLD, GREEN, PINK, SLATE,
};
use crate::game::{Game, ScreenState, Settings, Viewport};
use crate::input::MenuNavInput;
use macroquad::prelude::*;

pub const SETTINGS_ITEM_COUNT: usize = 13; // 12 settings (2 cols x 6 rows) + Back button

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseAction {
    Resume,
    OpenSettings,
    SaveAndQuit,
    AbortRun,
}

/// Pure pause-overlay navigation state machine (testable headless).
pub fn step_pause_nav(focus_idx: &mut usize, nav: MenuNavInput) -> Option<PauseAction> {
    if nav.back {
        return Some(PauseAction::Resume);
    }
    if *focus_idx >= 4 {
        *focus_idx = 0;
    }
    if nav.up {
        *focus_idx = if *focus_idx == 0 { 3 } else { *focus_idx - 1 };
    } else if nav.down {
        *focus_idx = (*focus_idx + 1) % 4;
    }
    if nav.confirm {
        return Some(match *focus_idx {
            0 => PauseAction::Resume,
            1 => PauseAction::OpenSettings,
            2 => PauseAction::SaveAndQuit,
            _ => PauseAction::AbortRun,
        });
    }
    None
}

/// Pure level-up 3-card draft navigation state machine (testable headless).
pub fn step_levelup_nav(focus_idx: &mut usize, nav: MenuNavInput) -> Option<usize> {
    if *focus_idx >= 3 {
        *focus_idx = 0;
    }
    if nav.up || nav.left || nav.prev_tab {
        *focus_idx = if *focus_idx == 0 { 2 } else { *focus_idx - 1 };
    } else if nav.down || nav.right || nav.next_tab {
        *focus_idx = (*focus_idx + 1) % 3;
    }
    if nav.confirm {
        Some(*focus_idx)
    } else {
        None
    }
}

/// Pure two-column Settings navigation & adjustment helper (returns `true` if user chose Back).
pub fn step_settings_nav(
    settings: &mut Settings,
    focus_idx: &mut usize,
    nav: MenuNavInput,
) -> bool {
    if nav.back {
        return true;
    }
    if *focus_idx >= SETTINGS_ITEM_COUNT {
        *focus_idx = 0;
    }

    let is_slider = matches!(*focus_idx, 0 | 1 | 2 | 9);
    if nav.up {
        if *focus_idx == 12 {
            *focus_idx = 5;
        } else if *focus_idx == 0 || *focus_idx == 6 {
            *focus_idx = 12;
        } else {
            *focus_idx -= 1;
        }
    } else if nav.down {
        if *focus_idx == 12 {
            *focus_idx = 0;
        } else if *focus_idx == 5 || *focus_idx == 11 {
            *focus_idx = 12;
        } else {
            *focus_idx += 1;
        }
    } else if nav.left {
        if is_slider {
            adjust_setting(settings, *focus_idx, -1);
        } else if (6..=11).contains(focus_idx) {
            *focus_idx -= 6;
        } else if *focus_idx < 6 {
            *focus_idx += 6;
        }
    } else if nav.right {
        if is_slider {
            adjust_setting(settings, *focus_idx, 1);
        } else if *focus_idx < 6 {
            *focus_idx += 6;
        } else if (6..=11).contains(focus_idx) {
            *focus_idx -= 6;
        }
    } else if nav.confirm {
        if *focus_idx == 12 {
            return true;
        }
        cycle_or_toggle_setting(settings, *focus_idx);
    }
    false
}

fn adjust_setting(settings: &mut Settings, idx: usize, delta: i32) {
    match idx {
        0 => settings.step_master_volume(delta),
        1 => settings.step_sfx_volume(delta),
        2 => settings.step_shake(delta),
        9 => settings.step_deadzone(delta),
        _ => cycle_or_toggle_setting(settings, idx),
    }
}

fn cycle_or_toggle_setting(settings: &mut Settings, idx: usize) {
    match idx {
        0 => settings.cycle_master_volume(),
        1 => settings.cycle_sfx_volume(),
        2 => settings.cycle_shake(),
        3 => settings.hit_stop = !settings.hit_stop,
        4 => settings.dmg_nums = !settings.dmg_nums,
        5 => settings.high_particles = !settings.high_particles,
        6 => settings.auto_aim = !settings.auto_aim,
        7 => settings.auto_fire = !settings.auto_fire,
        8 => settings.fixed_stick = !settings.fixed_stick,
        9 => settings.cycle_deadzone(),
        10 => settings.vibration = !settings.vibration,
        11 => settings.perf_hud = !settings.perf_hud,
        _ => {}
    }
}

pub fn render_paused(game: &mut Game, vp: Viewport, nav: MenuNavInput, pause_toggle: bool) {
    render_world(game, vp);
    let s = vp.scale;
    let vw = vp.vw;
    let vh = vp.vh;

    let mut action = if pause_toggle {
        Some(PauseAction::Resume)
    } else {
        step_pause_nav(&mut game.ui_focus.pause_idx, nav)
    };

    let show_focus = game.ui_focus.gamepad_nav_active;
    let fidx = game.ui_focus.pause_idx;

    draw_rectangle(0.0, 0.0, vp.sw, vp.sh, Color::from_rgba(4, 6, 11, 210));
    draw_centered_text("PAUZA", vw * 0.5 * s, vh * 0.19 * s, 36.0 * s, CYAN);

    let bw = 290.0;
    let bx = vw * 0.5 - bw * 0.5;
    let by = vh * 0.28;
    if ui_button_v_focused(
        Rect::new(bx, by, bw, 48.0),
        s,
        "WZNOW",
        None,
        CYAN,
        show_focus && fidx == 0,
    ) {
        action = Some(PauseAction::Resume);
    }
    if ui_button_v_focused(
        Rect::new(bx, by + 60.0, bw, 48.0),
        s,
        "OPCJE",
        None,
        GOLD,
        show_focus && fidx == 1,
    ) {
        action = Some(PauseAction::OpenSettings);
    }
    if ui_button_v_focused(
        Rect::new(bx, by + 120.0, bw, 48.0),
        s,
        "ZAPISZ I WYJDZ",
        Some("Zapisuje postep do wznowienia"),
        GREEN,
        show_focus && fidx == 2,
    ) {
        action = Some(PauseAction::SaveAndQuit);
    }
    if ui_button_v_focused(
        Rect::new(bx, by + 180.0, bw, 48.0),
        s,
        "PRZERWIJ LOT",
        Some("Konczy lot i zachowuje shardy"),
        PINK,
        show_focus && fidx == 3,
    ) {
        action = Some(PauseAction::AbortRun);
    }

    if let Some(act) = action {
        match act {
            PauseAction::Resume => game.resume_run(),
            PauseAction::OpenSettings => {
                game.prev_state = ScreenState::Paused;
                game.ui_focus.settings_idx = 0;
                game.state = ScreenState::Settings;
            }
            PauseAction::SaveAndQuit => game.save_and_quit_to_menu(),
            PauseAction::AbortRun => game.finish_run(false),
        }
    }
}

pub fn render_settings(game: &mut Game, vp: Viewport, nav: MenuNavInput) {
    let s = vp.scale;
    let vw = vp.vw;
    let vh = vp.vh;

    let mut close_requested =
        step_settings_nav(&mut game.settings, &mut game.ui_focus.settings_idx, nav);
    let show_focus = game.ui_focus.gamepad_nav_active;
    let fidx = game.ui_focus.settings_idx;

    draw_centered_text("OPCJE SYSTEMU", vw * 0.5 * s, 40.0 * s, 28.0 * s, CYAN);

    let total_w = (vw - vp.safe_x * 2.0).min(760.0);
    let gap_x = 14.0;
    let col_w = (total_w - gap_x) * 0.5;
    let left_x = vw * 0.5 - total_w * 0.5;
    let right_x = left_x + col_w + gap_x;

    // Column headers
    draw_centered_text(
        "AUDIO & EFEKTY",
        (left_x + col_w * 0.5) * s,
        68.0 * s,
        14.0 * s,
        GOLD,
    );
    draw_centered_text(
        "STEROWANIE & SYSTEM",
        (right_x + col_w * 0.5) * s,
        68.0 * s,
        14.0 * s,
        GOLD,
    );

    let start_y = 78.0;
    let row_h = ((vh - 160.0) / 6.0).clamp(44.0, 56.0);
    let btn_h = (row_h - 6.0).max(38.0);

    let st = game.settings;
    let shake_txt = match st.shake_intensity {
        0 => "OFF",
        1..=75 => "50%",
        _ => "100%",
    };
    let rows: [(String, &'static str, Color, bool); 12] = [
        // Left column: Audio + Visual & Feel (indices 0..6)
        (
            format!("GLOSNOSC GLOWNA: {}%", st.master_volume),
            "< -10% | +10% >",
            if st.master_volume > 0 { CYAN } else { SLATE },
            true,
        ),
        (
            format!("EFEKTY SFX: {}%", st.sfx_volume),
            "< -10% | +10% >",
            if st.sfx_volume > 0 { CYAN } else { SLATE },
            true,
        ),
        (
            format!("SHAKE EKRANU: {}", shake_txt),
            "< OFF / 50% / 100% >",
            if st.shake_intensity > 0 { CYAN } else { SLATE },
            true,
        ),
        (
            format!(
                "HIT-STOP: {}",
                if st.hit_stop { "WLACZONY" } else { "WYLACZONY" }
            ),
            "Pauza klatki przy trafieniu",
            if st.hit_stop { CYAN } else { SLATE },
            false,
        ),
        (
            format!(
                "OBRAZENIA: {}",
                if st.dmg_nums { "WLACZONE" } else { "WYLACZONE" }
            ),
            "Plywajace liczby trafien",
            if st.dmg_nums { CYAN } else { SLATE },
            false,
        ),
        (
            format!(
                "CZASTECZKI: {}",
                if st.high_particles {
                    "WYSOKIE (100%)"
                } else {
                    "NISKIE (50%)"
                }
            ),
            "Limit efektow na ekranie",
            if st.high_particles { GREEN } else { GOLD },
            false,
        ),
        // Right column: Controls + Perf HUD (indices 6..12)
        (
            format!(
                "AUTO-AIM: {}",
                if st.auto_aim { "WLACZONY" } else { "WYLACZONY" }
            ),
            "Automatyczne namierzanie wrogow",
            if st.auto_aim { CYAN } else { SLATE },
            false,
        ),
        (
            format!(
                "AUTO-FIRE: {}",
                if st.auto_fire {
                    "WLACZONY"
                } else {
                    "WYLACZONY"
                }
            ),
            "Wylacz, aby strzelac spustem/LPM",
            if st.auto_fire { CYAN } else { SLATE },
            false,
        ),
        (
            format!(
                "DRAZEK DOTYKU: {}",
                if st.fixed_stick { "STALY" } else { "PLYWAJACY" }
            ),
            "Pozycja lewego drazka dotykowego",
            CYAN,
            false,
        ),
        (
            format!("MARTWA STREFA: {}%", st.stick_deadzone),
            "< 5% - 30% (Dotyk + Pad) >",
            CYAN,
            true,
        ),
        (
            format!(
                "WIBRACJE PADA: {}",
                if st.vibration {
                    "WLACZONE"
                } else {
                    "WYLACZONE"
                }
            ),
            "Wibracje kontrolera przy wybuchach",
            if st.vibration { GREEN } else { SLATE },
            false,
        ),
        (
            format!(
                "PERF HUD: {}",
                if st.perf_hud { "WLACZONY" } else { "WYLACZONY" }
            ),
            "Licznik FPS, ms i liczby obiektow",
            if st.perf_hud { GOLD } else { SLATE },
            false,
        ),
    ];

    for (idx, (label, sub, col, is_slider)) in rows.iter().enumerate() {
        let col_idx = idx / 6;
        let row_idx = idx % 6;
        let rx = if col_idx == 0 { left_x } else { right_x };
        let ry = start_y + row_idx as f32 * row_h;
        let rect = Rect::new(rx, ry, col_w, btn_h);

        let hit_x = ui_button_hit_x(rect, s);
        let _ = ui_button_v_focused(rect, s, label, Some(sub), *col, show_focus && fidx == idx);
        if let Some(rel_x) = hit_x {
            game.ui_focus.settings_idx = idx;
            if *is_slider {
                if rel_x < 0.35 {
                    adjust_setting(&mut game.settings, idx, -1);
                } else if rel_x > 0.65 {
                    adjust_setting(&mut game.settings, idx, 1);
                } else {
                    cycle_or_toggle_setting(&mut game.settings, idx);
                }
            } else {
                cycle_or_toggle_setting(&mut game.settings, idx);
            }
            game.commit_settings();
        }
    }

    let back_w = total_w.min(340.0);
    let back_y = (start_y + 6.0 * row_h + 10.0).min(vh - 54.0);
    if ui_button_v_focused(
        Rect::new(vw * 0.5 - back_w * 0.5, back_y, back_w, 46.0),
        s,
        "ZAPISZ I WROC",
        None,
        GOLD,
        show_focus && fidx == 12,
    ) {
        close_requested = true;
    }

    if nav.any_pressed() {
        game.commit_settings();
    }

    if close_requested {
        game.commit_settings();
        game.touch.clear();
        game.state = game.prev_state;
    }
}

pub fn render_level_up(game: &mut Game, vp: Viewport, nav: MenuNavInput) {
    render_world(game, vp);
    let s = vp.scale;
    let vw = vp.vw;

    let mut chosen_slot = step_levelup_nav(&mut game.ui_focus.levelup_idx, nav);
    if is_key_pressed(KeyCode::Key1) {
        chosen_slot = Some(0);
    } else if is_key_pressed(KeyCode::Key2) {
        chosen_slot = Some(1);
    } else if is_key_pressed(KeyCode::Key3) {
        chosen_slot = Some(2);
    }

    let show_focus = game.ui_focus.gamepad_nav_active;
    let fidx = game.ui_focus.levelup_idx;

    draw_rectangle(0.0, 0.0, vp.sw, vp.sh, Color::from_rgba(4, 6, 11, 215));
    draw_centered_text("OVERCLOCK RDZENIA", vw * 0.5 * s, 68.0 * s, 32.0 * s, CYAN);
    let cw = (vw - vp.safe_x * 2.0).min(420.0);
    let choices = game.level_choices;
    for (i, opt) in choices.iter().enumerate() {
        let rect = Rect::new(vw * 0.5 - cw * 0.5, 110.0 + i as f32 * 86.0, cw, 72.0);
        if ui_button_v_focused(
            rect,
            s,
            opt.title,
            Some(opt.desc),
            CYAN,
            show_focus && fidx == i,
        ) {
            chosen_slot = Some(i);
        }
    }

    if let Some(slot) = chosen_slot {
        let id = choices[slot.min(2)].id;
        game.apply_upgrade(id);
    }
}

pub fn render_game_over(game: &mut Game, vp: Viewport, victory: bool, nav: MenuNavInput) {
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
    if ui_button_v_focused(
        Rect::new(vw * 0.5 - 150.0, stat_y + 92.0, 300.0, 54.0),
        s,
        "POWROT DO HANGARU",
        None,
        CYAN,
        game.ui_focus.gamepad_nav_active,
    ) || nav.confirm
        || nav.back
    {
        game.touch.clear();
        game.ui_focus.menu_idx = 0;
        game.state = ScreenState::Menu;
    }
}
