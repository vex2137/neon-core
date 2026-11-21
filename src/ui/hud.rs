//! Zero-allocation in-game HUD: bars, wave counter, minimap, dash cooldown arc, and virtual sticks.

use super::{draw_centered_text, draw_right_text, CYAN, GOLD, PINK, PURPLE};
use crate::effects::TextBuf;
use crate::enemies::EnemyKind;
use crate::game::{Game, Viewport, ARENA_RADIUS};
use crate::input::{
    dash_button_center, left_stick_rest_center, pause_button_rect, DASH_BUTTON_RADIUS,
    STICK_MAX_RADIUS,
};
use crate::progression::{combo_xp_multiplier, WAVE_DURATION};
use macroquad::prelude::*;
use std::fmt::Write;

pub fn render_hud(game: &Game, vp: Viewport) {
    let s = vp.scale;
    let vw = vp.vw;
    let sx_pad = vp.safe_x;
    let compact = vw < 700.0;
    let pause_rect = pause_button_rect(vp);

    // Left: HULL, optional Plasma SHIELD, & LVL/XP bars
    let bar_w = (vw * if compact { 0.24 } else { 0.22 }).clamp(125.0, 195.0);
    let lbl_sz = if compact { 15.0 } else { 17.0 };

    let mut hull_buf = TextBuf::<32>::new();
    let _ = write!(
        &mut hull_buf,
        "HULL {}/{}",
        game.player.hp.ceil() as i32,
        game.player.max_hp as i32
    );
    draw_text(hull_buf.as_str(), sx_pad * s, 24.0 * s, lbl_sz * s, WHITE);
    draw_rectangle(
        sx_pad * s,
        30.0 * s,
        bar_w * s,
        9.0 * s,
        Color::from_rgba(255, 255, 255, 25),
    );
    draw_rectangle(
        sx_pad * s,
        30.0 * s,
        bar_w * (game.player.hp / game.player.max_hp.max(1.0)).clamp(0.0, 1.0) * s,
        9.0 * s,
        PINK,
    );
    draw_rectangle_lines(sx_pad * s, 30.0 * s, bar_w * s, 9.0 * s, 1.2 * s, CYAN);

    let mut left_y = 43.0;
    if game.player.max_shield > 0.0 {
        let lcyan = Color::from_rgba(125, 249, 255, 255);
        draw_rectangle(
            sx_pad * s,
            left_y * s,
            bar_w * s,
            6.5 * s,
            Color::from_rgba(255, 255, 255, 20),
        );
        draw_rectangle(
            sx_pad * s,
            left_y * s,
            bar_w * (game.player.shield / game.player.max_shield).clamp(0.0, 1.0) * s,
            6.5 * s,
            lcyan,
        );
        draw_rectangle_lines(
            sx_pad * s,
            left_y * s,
            bar_w * s,
            6.5 * s,
            1.0 * s,
            Color::from_rgba(125, 249, 255, 190),
        );
        left_y += 10.5;
    }

    let lvl_lbl_y = left_y + 13.0;
    let lvl_bar_y = left_y + 19.0;
    let mut lvl_buf = TextBuf::<24>::new();
    let _ = write!(&mut lvl_buf, "LVL {}", game.level);
    draw_text(
        lvl_buf.as_str(),
        sx_pad * s,
        lvl_lbl_y * s,
        lbl_sz * s,
        WHITE,
    );
    draw_rectangle(
        sx_pad * s,
        lvl_bar_y * s,
        bar_w * s,
        9.0 * s,
        Color::from_rgba(255, 255, 255, 25),
    );
    draw_rectangle(
        sx_pad * s,
        lvl_bar_y * s,
        bar_w * (game.xp / game.next_xp.max(1.0)).clamp(0.0, 1.0) * s,
        9.0 * s,
        CYAN,
    );
    draw_rectangle_lines(sx_pad * s, lvl_bar_y * s, bar_w * s, 9.0 * s, 1.2 * s, CYAN);

    // Subtle red edge damage flash
    if game.effects.hurt_flash > 0.01 {
        let ha = (game.effects.hurt_flash * 0.55 * 255.0) as u8;
        draw_rectangle_lines(
            0.0,
            0.0,
            vp.sw,
            vp.sh,
            18.0 * s,
            Color::from_rgba(255, 58, 92, ha),
        );
    }

    // Right: SHARDS, DIFF/FPS, COMBO & OVERDRIVE
    let right_edge = pause_rect.x - 12.0;
    let shard_sz = if compact { 15.0 } else { 18.0 };
    let mut shard_buf = TextBuf::<32>::new();
    let _ = write!(&mut shard_buf, "SHARDS: {}", game.run_shards);
    let shard_w_phys = draw_right_text(
        shard_buf.as_str(),
        right_edge * s,
        24.0 * s,
        shard_sz * s,
        GOLD,
    );
    let shard_w = shard_w_phys / s;

    let diff = game.difficulty();
    let mut meta_buf = TextBuf::<32>::new();
    if diff.id > 0 {
        let _ = write!(&mut meta_buf, "{} | {} FPS", diff.name, get_fps());
    } else {
        let _ = write!(&mut meta_buf, "{} FPS", get_fps());
    }
    draw_right_text(
        meta_buf.as_str(),
        right_edge * s,
        43.0 * s,
        (if compact { 12.5 } else { 14.5 }) * s,
        if diff.id > 0 { diff.color() } else { LIGHTGRAY },
    );

    let mut right_row_y = 60.0;
    if game.combo >= 3 {
        let mult = combo_xp_multiplier(game.combo);
        let mut combo_buf = TextBuf::<32>::new();
        let _ = write!(&mut combo_buf, "COMBO {} ({:.2}x)", game.combo, mult);
        draw_right_text(
            combo_buf.as_str(),
            right_edge * s,
            right_row_y * s,
            (if compact { 13.0 } else { 14.5 }) * s,
            CYAN,
        );
        right_row_y += 17.0;
    }
    if game.player.overdrive_timer > 0.0 {
        let mut od_buf = TextBuf::<32>::new();
        let _ = write!(&mut od_buf, "OVERDRIVE {:.1}s", game.player.overdrive_timer);
        draw_right_text(
            od_buf.as_str(),
            right_edge * s,
            right_row_y * s,
            (if compact { 13.0 } else { 14.5 }) * s,
            GOLD,
        );
    }

    // Center: WAVE dynamically centered in free gap between Left and Right blocks
    let left_end = sx_pad + bar_w + 12.0;
    let right_start = right_edge - shard_w - 12.0;
    let free_gap = right_start - left_end;

    let mut wave_buf = TextBuf::<24>::new();
    let _ = write!(&mut wave_buf, "WAVE {}", game.wave);
    let mut timer_buf = TextBuf::<32>::new();
    let _ = write!(
        &mut timer_buf,
        "{}s{}",
        (WAVE_DURATION - game.wave_timer).ceil().max(1.0) as i32,
        game.wave_event.tag()
    );

    if free_gap >= 110.0 {
        let wave_cx = left_end + free_gap * 0.5;
        draw_centered_text(
            wave_buf.as_str(),
            wave_cx * s,
            26.0 * s,
            (if compact { 19.0 } else { 23.0 }) * s,
            PINK,
        );
        draw_centered_text(
            timer_buf.as_str(),
            wave_cx * s,
            45.0 * s,
            (if compact { 13.0 } else { 15.0 }) * s,
            LIGHTGRAY,
        );
    } else {
        let mut combined = TextBuf::<48>::new();
        let _ = write!(
            &mut combined,
            "{} | {}",
            wave_buf.as_str(),
            timer_buf.as_str()
        );
        draw_centered_text(
            combined.as_str(),
            vw * 0.5 * s,
            (lvl_bar_y + 26.0) * s,
            16.0 * s,
            PINK,
        );
    }

    // Wave Event / Beacon announcement banner
    if game.banner.timer > 0.0 {
        let alpha = (game.banner.timer / 0.75).clamp(0.0, 1.0);
        let by = if free_gap >= 110.0 {
            64.0
        } else {
            lvl_bar_y + 42.0
        };
        let bw = (vw - sx_pad * 2.0 - 150.0).min(430.0);
        if bw > 160.0 {
            let bx = vw * 0.5 - bw * 0.5;
            draw_rectangle(
                bx * s,
                by * s,
                bw * s,
                44.0 * s,
                Color::from_rgba(6, 12, 24, (0.78 * alpha * 255.0) as u8),
            );
            let mut bc = game.banner.color;
            bc.a = 0.85 * alpha;
            draw_rectangle_lines(bx * s, by * s, bw * s, 44.0 * s, 1.6 * s, bc);
            bc.a = alpha;
            draw_centered_text(
                game.banner.title.as_str(),
                vw * 0.5 * s,
                (by + 19.0) * s,
                (if compact { 14.0 } else { 16.0 }) * s,
                bc,
            );
            draw_centered_text(
                game.banner.subtitle.as_str(),
                vw * 0.5 * s,
                (by + 35.0) * s,
                (if compact { 11.5 } else { 12.5 }) * s,
                Color::from_rgba(226, 232, 240, (alpha * 255.0) as u8),
            );
        }
    }

    // Pause button
    draw_rectangle(
        pause_rect.x * s,
        pause_rect.y * s,
        pause_rect.w * s,
        pause_rect.h * s,
        Color::from_rgba(10, 16, 30, 225),
    );
    draw_rectangle_lines(
        pause_rect.x * s,
        pause_rect.y * s,
        pause_rect.w * s,
        pause_rect.h * s,
        1.8 * s,
        CYAN,
    );
    draw_centered_text(
        "||",
        (pause_rect.x + pause_rect.w * 0.5) * s,
        (pause_rect.y + pause_rect.h * 0.67) * s,
        (if compact { 17.0 } else { 20.0 }) * s,
        CYAN,
    );

    // Compact tactical minimap
    let map_r = if compact { 30.0 } else { 36.0 };
    let map_cx = sx_pad + map_r;
    let map_cy = lvl_bar_y + 22.0 + map_r;
    draw_circle(
        map_cx * s,
        map_cy * s,
        map_r * s,
        Color::from_rgba(6, 12, 24, 190),
    );
    draw_circle_lines(
        map_cx * s,
        map_cy * s,
        map_r * s,
        1.6 * s,
        Color::from_rgba(255, 42, 109, 175),
    );
    let map_scale = (map_r - 3.5) / ARENA_RADIUS;
    if game.beacon.active {
        let bmx = (map_cx + game.beacon.pos.x * map_scale) * s;
        let bmy = (map_cy + game.beacon.pos.y * map_scale) * s;
        draw_rectangle(bmx - 2.8 * s, bmy - 2.8 * s, 5.6 * s, 5.6 * s, GOLD);
    }
    for e in &game.enemies {
        if !e.alive {
            continue;
        }
        let mx = (map_cx + e.pos.x * map_scale) * s;
        let my = (map_cy + e.pos.y * map_scale) * s;
        let c = if e.kind == EnemyKind::Boss {
            PURPLE
        } else if e.elite {
            WHITE
        } else {
            Color::from_rgba(255, 42, 109, 195)
        };
        let dot = (if e.kind == EnemyKind::Boss {
            3.4
        } else if e.elite {
            2.5
        } else {
            1.7
        }) * s;
        draw_rectangle(mx - dot * 0.5, my - dot * 0.5, dot, dot, c);
    }
    draw_circle(
        (map_cx + game.player.pos.x * map_scale) * s,
        (map_cy + game.player.pos.y * map_scale) * s,
        2.8 * s,
        if game.player.overdrive_timer > 0.0 {
            GOLD
        } else {
            CYAN
        },
    );

    // Enlarged right-side Dash button with radial cooldown arc
    let dc = dash_button_center(vp);
    let dash_r = DASH_BUTTON_RADIUS;
    let dash_ready = game.player.dash_cd <= 0.0;
    let dash_col = if dash_ready {
        CYAN
    } else {
        Color::from_rgba(95, 125, 165, 210)
    };
    let fill_col = if dash_ready {
        Color::from_rgba(5, 217, 232, 36)
    } else {
        Color::from_rgba(8, 14, 28, 185)
    };
    draw_circle(dc.x * s, dc.y * s, dash_r * s, fill_col);
    draw_circle_lines(
        dc.x * s,
        dc.y * s,
        dash_r * s,
        2.0 * s,
        Color::from_rgba(70, 95, 130, 140),
    );

    if dash_ready {
        // Single circle draw call when ready (avoids 32 segment draw calls + 64 trig ops).
        draw_circle_lines(dc.x * s, dc.y * s, (dash_r - 2.5) * s, 4.0 * s, CYAN);
        let pulse = (get_time() as f32 * 4.5).sin() * 2.5;
        draw_circle_lines(
            dc.x * s,
            dc.y * s,
            (dash_r + 5.0 + pulse) * s,
            1.5 * s,
            Color::from_rgba(5, 217, 232, 95),
        );
    } else {
        let progress =
            (1.0_f32 - game.player.dash_cd / game.player.dash_max_cd.max(0.01)).clamp(0.0, 1.0);
        if progress > 0.01 {
            let segs = 28;
            let max_a = progress * std::f32::consts::TAU;
            let start_a = -std::f32::consts::FRAC_PI_2;
            let arc_r = (dash_r - 2.5) * s;
            let arc_col = Color::from_rgba(5, 217, 232, 220);
            for i in 0..segs {
                let a0 = start_a + (i as f32 / segs as f32) * max_a;
                let a1 = start_a + ((i + 1) as f32 / segs as f32) * max_a;
                let p0 = vec2(dc.x * s + a0.cos() * arc_r, dc.y * s + a0.sin() * arc_r);
                let p1 = vec2(dc.x * s + a1.cos() * arc_r, dc.y * s + a1.sin() * arc_r);
                draw_line(p0.x, p0.y, p1.x, p1.y, 3.5 * s, arc_col);
            }
        }
    }

    draw_centered_text("DASH", dc.x * s, (dc.y - 3.0) * s, 19.0 * s, dash_col);
    if dash_ready {
        draw_centered_text("READY", dc.x * s, (dc.y + 17.0) * s, 13.5 * s, dash_col);
    } else {
        let mut cd_buf = TextBuf::<16>::new();
        let _ = write!(&mut cd_buf, "{:.1}s", game.player.dash_cd);
        draw_centered_text(
            cd_buf.as_str(),
            dc.x * s,
            (dc.y + 17.0) * s,
            13.5 * s,
            dash_col,
        );
    }

    // Dynamic Left Movement Joystick (with subtle resting thumb guide)
    let rest = left_stick_rest_center(vp);
    if game.touch.left_stick.touch_id.is_none() {
        let rc = rest * s;
        let rr = 48.0 * s;
        draw_circle(rc.x, rc.y, rr, Color::from_rgba(5, 217, 232, 14));
        draw_circle_lines(rc.x, rc.y, rr, 2.0 * s, Color::from_rgba(5, 217, 232, 72));
        draw_circle(rc.x, rc.y, 15.0 * s, Color::from_rgba(5, 217, 232, 48));
        let tick = Color::from_rgba(5, 217, 232, 65);
        draw_line(
            rc.x - rr + 6.0 * s,
            rc.y,
            rc.x - rr + 14.0 * s,
            rc.y,
            1.5 * s,
            tick,
        );
        draw_line(
            rc.x + rr - 14.0 * s,
            rc.y,
            rc.x + rr - 6.0 * s,
            rc.y,
            1.5 * s,
            tick,
        );
        draw_line(
            rc.x,
            rc.y - rr + 6.0 * s,
            rc.x,
            rc.y - rr + 14.0 * s,
            1.5 * s,
            tick,
        );
        draw_line(
            rc.x,
            rc.y + rr - 14.0 * s,
            rc.x,
            rc.y + rr - 6.0 * s,
            1.5 * s,
            tick,
        );
    } else {
        let st = game.touch.left_stick.start * s;
        let cur = game.touch.left_stick.cur * s;
        draw_circle(
            st.x,
            st.y,
            STICK_MAX_RADIUS * s,
            Color::from_rgba(5, 217, 232, 24),
        );
        draw_circle_lines(st.x, st.y, STICK_MAX_RADIUS * s, 2.4 * s, CYAN);
        draw_line(
            st.x,
            st.y,
            cur.x,
            cur.y,
            2.5 * s,
            Color::from_rgba(5, 217, 232, 130),
        );
        draw_circle(cur.x, cur.y, 20.0 * s, Color::from_rgba(5, 217, 232, 220));
        draw_circle_lines(cur.x, cur.y, 20.0 * s, 2.0 * s, WHITE);
    }

    if game.touch.right_stick.touch_id.is_some() {
        let st = game.touch.right_stick.start * s;
        let cur = game.touch.right_stick.cur * s;
        draw_circle(
            st.x,
            st.y,
            STICK_MAX_RADIUS * s,
            Color::from_rgba(255, 42, 109, 22),
        );
        draw_circle_lines(st.x, st.y, STICK_MAX_RADIUS * s, 2.4 * s, PINK);
        draw_line(
            st.x,
            st.y,
            cur.x,
            cur.y,
            2.5 * s,
            Color::from_rgba(255, 42, 109, 130),
        );
        draw_circle(cur.x, cur.y, 20.0 * s, Color::from_rgba(255, 42, 109, 220));
    }
}
