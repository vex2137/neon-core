//! Hangar menu: permanent shop tabs, ship selector, difficulty selector, mid-run resume, and gamepad nav.

use super::{draw_centered_text, ui_button_v_focused, CYAN, GOLD, GREEN, PINK, PURPLE, SLATE};
use crate::game::{Game, ScreenState, Viewport};
use crate::input::MenuNavInput;
use crate::player::{ship_by_id, SHIPS};
use crate::progression::{DIFFICULTIES, SHOP_ITEMS};
use macroquad::prelude::*;

/// Pure menu focus navigation step (testable headless without Macroquad window).
pub fn step_menu_focus(
    shop_tab: &mut u8,
    focus_idx: &mut usize,
    nav: MenuNavInput,
    has_saved_run: bool,
) -> Option<usize> {
    if nav.prev_tab {
        *shop_tab = if *shop_tab == 0 { 3 } else { *shop_tab - 1 };
        *focus_idx = 0;
        return None;
    }
    if nav.next_tab {
        *shop_tab = (*shop_tab + 1) % 4;
        *focus_idx = 0;
        return None;
    }

    let card_count: usize = if *shop_tab == 3 { 4 } else { 5 };
    let diff_start = card_count;
    let action_start = diff_start + 3;
    let action_count: usize = if has_saved_run { 3 } else { 2 };
    let total = action_start + action_count;

    if *focus_idx >= total {
        *focus_idx = total - 1;
    }

    if nav.back {
        // Jump quickly to bottom launch/resume row.
        *focus_idx = action_start;
        return None;
    }

    if nav.left {
        *focus_idx = if *focus_idx == 0 {
            total - 1
        } else {
            *focus_idx - 1
        };
    } else if nav.right {
        *focus_idx = (*focus_idx + 1) % total;
    } else if nav.up {
        if *focus_idx >= action_start {
            *focus_idx = diff_start + (*focus_idx - action_start).min(2);
        } else if *focus_idx >= diff_start {
            *focus_idx = card_count.saturating_sub(1);
        } else if *focus_idx >= 2 {
            *focus_idx -= 2;
        } else {
            *focus_idx = action_start;
        }
    } else if nav.down {
        if *focus_idx < card_count {
            let next = *focus_idx + 2;
            if next < card_count {
                *focus_idx = next;
            } else {
                *focus_idx = diff_start;
            }
        } else if *focus_idx < action_start {
            *focus_idx = action_start + (*focus_idx - diff_start).min(action_count - 1);
        } else {
            *focus_idx = 0;
        }
    }

    if nav.confirm {
        Some(*focus_idx)
    } else {
        None
    }
}

pub fn render_menu(game: &mut Game, vp: Viewport, nav: MenuNavInput) {
    let s = vp.scale;
    let vw = vp.vw;
    let vh = vp.vh;
    let compact = vw < 720.0 || vh < 500.0;
    let has_saved_run = game.saved_run_summary.is_some();

    let activated_focus = step_menu_focus(
        &mut game.shop_tab,
        &mut game.ui_focus.menu_idx,
        nav,
        has_saved_run,
    );
    let show_focus = game.ui_focus.gamepad_nav_active;
    let focus_idx = game.ui_focus.menu_idx;
    let sfx_gain = game.settings.effective_sfx_gain();

    let base_badge = if cfg!(feature = "demo") {
        "RUST WASM DEMO (MAX WAVE 8)"
    } else {
        "NATIVE RUST ARM64 | HANGAR & SKLEP"
    };
    let badge = if game.meta.prestige_lvl > 0 {
        format!("{}  |  PRESTIZ *{}", base_badge, game.meta.prestige_lvl)
    } else {
        base_badge.to_string()
    };
    draw_centered_text(
        &badge,
        vw * 0.5 * s,
        (if compact { 20.0 } else { 24.0 }) * s,
        (if compact { 12.0 } else { 13.5 }) * s,
        PINK,
    );
    draw_centered_text(
        "NEON CORE",
        vw * 0.5 * s,
        (if compact { 46.0 } else { 56.0 }) * s,
        (if compact { 30.0 } else { 36.0 }) * s,
        CYAN,
    );
    let sub_hdr = if game.meta.best_wave > 0 {
        format!(
            "SHARDS: {}  |  REKORD: FALA {} ({} KILLS)",
            game.meta.shards, game.meta.best_wave, game.meta.best_kills
        )
    } else {
        format!("SHARDS: {}", game.meta.shards)
    };
    draw_centered_text(
        &sub_hdr,
        vw * 0.5 * s,
        (if compact { 66.0 } else { 77.0 }) * s,
        (if compact { 14.0 } else { 15.5 }) * s,
        GOLD,
    );

    // 4 Hangar tabs: BOJOWE | SURVIVAL | ENDGAME | STATKI (LB/RB switches tabs on gamepad)
    let tab_y = if compact { 76.0 } else { 88.0 };
    let tab_gap = 8.0;
    let total_tab_w = (vw - vp.safe_x * 2.0).min(640.0);
    let tab_w = (total_tab_w - tab_gap * 3.0) / 4.0;
    let tab_h = if compact { 30.0 } else { 34.0 };
    let tab_start_x = vw * 0.5 - total_tab_w * 0.5;
    let tab_defs: [(u8, &str, Color); 4] = [
        (0, if compact { "BOJOWE" } else { "BOJOWE (5)" }, CYAN),
        (1, if compact { "SURVIVAL" } else { "SURVIVAL (5)" }, GREEN),
        (2, if compact { "ENDGAME" } else { "ENDGAME (5)" }, GOLD),
        (3, if compact { "STATKI" } else { "STATKI (4)" }, PURPLE),
    ];
    for (t_idx, (tid, lbl, col)) in tab_defs.iter().enumerate() {
        let rx = tab_start_x + t_idx as f32 * (tab_w + tab_gap);
        let active = game.shop_tab == *tid;
        if ui_button_v_focused(
            Rect::new(rx, tab_y, tab_w, tab_h),
            s,
            lbl,
            None,
            if active { *col } else { SLATE },
            false,
        ) {
            game.shop_tab = *tid;
            game.ui_focus.menu_idx = 0;
        }
    }

    let cols = if vw >= 620.0 { 2 } else { 1 };
    let start_y = tab_y + tab_h + 10.0;
    let active_tab = game.shop_tab;
    let card_count: usize = if active_tab == 3 { 4 } else { 5 };

    let cards_end_y: f32 = if active_tab == 3 {
        let card_w = if cols == 2 {
            ((vw - vp.safe_x * 2.0 - 12.0) * 0.5).min(340.0)
        } else {
            (vw - vp.safe_x * 2.0).min(400.0)
        };
        let card_h = if compact { 58.0 } else { 70.0 };
        let gap_y = 10.0;
        let start_x = vw * 0.5 - (cols as f32 * card_w + (cols - 1) as f32 * 12.0) * 0.5;

        for (i, ship) in SHIPS.iter().enumerate() {
            let r = i / cols;
            let c = i % cols;
            let rx = start_x + c as f32 * (card_w + 12.0);
            let ry = start_y + r as f32 * (card_h + gap_y);

            let unlocked = (game.meta.unlocked_ships_mask & (1_u32 << ship.id)) != 0;
            let selected = game.meta.selected_ship == ship.id;
            let can_buy = !unlocked && game.meta.shards >= ship.cost;
            let border = if selected {
                ship.color()
            } else if unlocked {
                CYAN
            } else if can_buy {
                GOLD
            } else {
                SLATE
            };
            let title = if selected {
                format!("{} [WYBRANY]", ship.name)
            } else if unlocked {
                format!("{} [WYBIERZ]", ship.name)
            } else {
                format!("{} [{} SHARDS]", ship.name, ship.cost)
            };

            let clicked = ui_button_v_focused(
                Rect::new(rx, ry, card_w, card_h),
                s,
                &title,
                Some(ship.perk),
                border,
                show_focus && focus_idx == i,
            );
            if clicked || activated_focus == Some(i) {
                if unlocked {
                    game.meta.selected_ship = ship.id;
                    if game.persist_saves {
                        game.meta.save();
                    }
                    game.sounds.play(&game.sounds.hit, sfx_gain, 0.55);
                } else if can_buy {
                    game.meta.shards -= ship.cost;
                    game.meta.unlocked_ships_mask |= 1_u32 << ship.id;
                    game.meta.selected_ship = ship.id;
                    if game.persist_saves {
                        game.meta.save();
                    }
                    game.sounds.play(&game.sounds.lvlup, sfx_gain, 0.9);
                }
            }
        }
        let rows = SHIPS.len().div_ceil(cols);
        start_y + rows as f32 * (card_h + gap_y)
    } else {
        let card_w = if cols == 2 {
            ((vw - vp.safe_x * 2.0 - 12.0) * 0.5).min(340.0)
        } else {
            (vw - vp.safe_x * 2.0).min(400.0)
        };
        let card_h = if compact { 46.0 } else { 54.0 };
        let gap_y = 8.0;
        let start_x = vw * 0.5 - (cols as f32 * card_w + (cols - 1) as f32 * 12.0) * 0.5;

        let mut visible_idx = 0_usize;
        for item in SHOP_ITEMS.iter().filter(|it| it.tab == active_tab) {
            let r = visible_idx / cols;
            let c = visible_idx % cols;
            let is_last_odd = cols == 2 && visible_idx == 4;
            let rx = if is_last_odd {
                vw * 0.5 - card_w * 0.5
            } else {
                start_x + c as f32 * (card_w + 12.0)
            };
            let ry = start_y + r as f32 * (card_h + gap_y);
            let idx = visible_idx;
            visible_idx += 1;

            let lvl = game.meta.get_lvl(item.id);
            let maxed = lvl >= item.max_lvl;
            let cost = item.next_cost(lvl);
            let can_buy = !maxed && game.meta.shards >= cost;
            let border = if maxed {
                GREEN
            } else if can_buy {
                GOLD
            } else {
                SLATE
            };
            let title = if maxed {
                format!("{} [MAX]", item.name)
            } else {
                format!("{} [{}/{}]", item.name, lvl, item.max_lvl)
            };
            let sub = if maxed {
                format!("{} | MAKSIMUM", item.bonus)
            } else {
                format!("{} | {} SHARDS", item.bonus, cost)
            };

            let clicked = ui_button_v_focused(
                Rect::new(rx, ry, card_w, card_h),
                s,
                &title,
                Some(&sub),
                border,
                show_focus && focus_idx == idx,
            );
            if (clicked || activated_focus == Some(idx)) && can_buy {
                game.meta.shards -= cost;
                game.meta.inc_lvl(item.id);
                if game.persist_saves {
                    game.meta.save();
                }
                game.sounds.play(&game.sounds.lvlup, sfx_gain, 0.8);
            }
        }
        let rows = visible_idx.div_ceil(cols);
        start_y + rows as f32 * (card_h + gap_y)
    };

    // Difficulty selector row
    let diff_start = card_count;
    let diff_y = (cards_end_y + 6.0).min(vh - 104.0);
    let diff_gap = 8.0;
    let diff_total_w = (vw - vp.safe_x * 2.0).min(560.0);
    let diff_w = (diff_total_w - diff_gap * 2.0) / 3.0;
    let diff_h = 34.0;
    let diff_start_x = vw * 0.5 - diff_total_w * 0.5;
    for (d_idx, df) in DIFFICULTIES.iter().enumerate() {
        let idx = diff_start + d_idx;
        let active = game.meta.difficulty == df.id;
        let rx = diff_start_x + d_idx as f32 * (diff_w + diff_gap);
        let clicked = ui_button_v_focused(
            Rect::new(rx, diff_y, diff_w, diff_h),
            s,
            df.short_tag,
            None,
            if active { df.color() } else { SLATE },
            show_focus && focus_idx == idx,
        );
        if clicked || activated_focus == Some(idx) {
            game.meta.difficulty = df.id;
            if game.persist_saves {
                game.meta.save();
            }
        }
    }

    // Bottom action row: [WZNOW LOT] (when saved run exists) | NEW RUN / START | OPCJE
    let action_start = diff_start + 3;
    let btn_y = (diff_y + diff_h + 10.0).min(vh - 58.0);
    let btn_h = 46.0;
    let active_ship = game.selected_ship();

    if let Some(summary) = game.saved_run_summary {
        let total_w = (vw - vp.safe_x * 2.0).min(640.0);
        let gap = 10.0;
        let resume_w = (total_w - gap * 2.0) * 0.44;
        let new_w = (total_w - gap * 2.0) * 0.34;
        let opt_w = (total_w - gap * 2.0) * 0.22;
        let x0 = vw * 0.5 - total_w * 0.5;

        let saved_ship = ship_by_id(summary.ship_id);
        let saved_diff = DIFFICULTIES[(summary.difficulty as usize).min(DIFFICULTIES.len() - 1)];
        let resume_sub = format!(
            "{} | FALA {} | {}",
            saved_ship.short_name, summary.wave, saved_diff.name
        );

        let resume_clicked = ui_button_v_focused(
            Rect::new(x0, btn_y, resume_w, btn_h),
            s,
            "WZNOW LOT",
            Some(&resume_sub),
            GREEN,
            show_focus && focus_idx == action_start,
        );
        if resume_clicked || activated_focus == Some(action_start) {
            if !game.resume_saved_run() {
                game.start_run(vp);
            }
            return;
        }

        let new_lbl = format!("NOWY: {}", active_ship.short_name);
        let new_clicked = ui_button_v_focused(
            Rect::new(x0 + resume_w + gap, btn_y, new_w, btn_h),
            s,
            &new_lbl,
            None,
            active_ship.color(),
            show_focus && focus_idx == action_start + 1,
        );
        if new_clicked || activated_focus == Some(action_start + 1) {
            game.start_run(vp);
            return;
        }

        let opt_clicked = ui_button_v_focused(
            Rect::new(x0 + resume_w + gap + new_w + gap, btn_y, opt_w, btn_h),
            s,
            "OPCJE",
            None,
            GOLD,
            show_focus && focus_idx == action_start + 2,
        );
        if opt_clicked || activated_focus == Some(action_start + 2) {
            game.prev_state = ScreenState::Menu;
            game.ui_focus.settings_idx = 0;
            game.state = ScreenState::Settings;
        }
    } else {
        let btn_w = ((vw - vp.safe_x * 2.0 - 16.0) * 0.5).min(220.0);
        let start_lbl = format!("START: {}", active_ship.short_name);

        let start_clicked = ui_button_v_focused(
            Rect::new(vw * 0.5 - btn_w - 8.0, btn_y, btn_w, btn_h),
            s,
            &start_lbl,
            None,
            active_ship.color(),
            show_focus && focus_idx == action_start,
        );
        if start_clicked || activated_focus == Some(action_start) {
            game.start_run(vp);
            return;
        }

        let opt_clicked = ui_button_v_focused(
            Rect::new(vw * 0.5 + 8.0, btn_y, btn_w, btn_h),
            s,
            "OPCJE",
            None,
            GOLD,
            show_focus && focus_idx == action_start + 1,
        );
        if opt_clicked || activated_focus == Some(action_start + 1) {
            game.prev_state = ScreenState::Menu;
            game.ui_focus.settings_idx = 0;
            game.state = ScreenState::Settings;
        }
    }
}
