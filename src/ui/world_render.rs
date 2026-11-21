//! World-space rendering with high-DPI virtual camera, frustum culling, and zero-alloc ship geometry.

use super::draw_centered_text;
use crate::enemies::EnemyKind;
use crate::game::{Game, Viewport, ARENA_RADIUS};
use macroquad::prelude::*;

/// Computes stack-allocated hull vertices for the 4 playable ship archetypes (zero heap allocation).
#[inline]
fn ship_hull_vertices(ship_id: u32, p: Vec2, aim_angle: f32) -> ([Vec2; 7], usize) {
    let fwd = Vec2::from_angle(aim_angle);
    let right_v = vec2(-fwd.y, fwd.x);
    match ship_id {
        1 => (
            [
                p + fwd * 24.0,
                p - fwd * 4.0 + right_v * 16.0,
                p - fwd * 15.0 + right_v * 7.0,
                p - fwd * 8.0,
                p - fwd * 15.0 - right_v * 7.0,
                p - fwd * 4.0 - right_v * 16.0,
                Vec2::ZERO,
            ],
            6,
        ),
        2 => (
            [
                p + fwd * 19.0 + right_v * 6.0,
                p + fwd * 6.0 + right_v * 18.0,
                p - fwd * 15.0 + right_v * 14.0,
                p - fwd * 18.0,
                p - fwd * 15.0 - right_v * 14.0,
                p + fwd * 6.0 - right_v * 18.0,
                p + fwd * 19.0 - right_v * 6.0,
            ],
            7,
        ),
        3 => (
            [
                p + fwd * 22.0,
                p + fwd * 4.0 + right_v * 13.0,
                p - fwd * 17.0 + right_v * 15.0,
                p - fwd * 6.0,
                p - fwd * 17.0 - right_v * 15.0,
                p + fwd * 4.0 - right_v * 13.0,
                Vec2::ZERO,
            ],
            6,
        ),
        _ => (
            [
                p + fwd * 20.0,
                p + Vec2::from_angle(aim_angle + 2.4) * 15.0,
                p - fwd * 7.0,
                p + Vec2::from_angle(aim_angle - 2.4) * 15.0,
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
            ],
            4,
        ),
    }
}

pub fn render_world(game: &Game, vp: Viewport) {
    let vw = vp.vw;
    let vh = vp.vh;
    let shake = game.effects.shake;
    let (sx, sy) = if shake > 0.0 {
        // Deterministic-free fast visual jitter derived from frame time phase
        let t = get_time() as f32;
        ((t * 73.1).sin() * shake, (t * 97.7).cos() * shake)
    } else {
        (0.0, 0.0)
    };

    // Parallax starfield scaled to high-DPI virtual viewport
    for star in &game.stars {
        let x = ((star.pos.x - game.player.pos.x * star.depth).rem_euclid(3200.0)) - 800.0;
        let y = ((star.pos.y - game.player.pos.y * star.depth).rem_euclid(3200.0)) - 800.0;
        if x >= -5.0 && x <= vw + 5.0 && y >= -5.0 && y <= vh + 5.0 {
            draw_rectangle(
                x * vp.scale,
                y * vp.scale,
                star.size * vp.scale,
                star.size * vp.scale,
                Color::from_rgba(140, 210, 255, 110),
            );
        }
    }

    // Virtual world camera zoomed by viewport scale
    let cam_rect = Rect::new(
        game.player.pos.x - vw * 0.5 + sx,
        game.player.pos.y - vh * 0.5 + sy,
        vw,
        vh,
    );
    let vmin_x = cam_rect.x - 60.0;
    let vmax_x = cam_rect.x + vw + 60.0;
    let vmin_y = cam_rect.y - 60.0;
    let vmax_y = cam_rect.y + vh + 60.0;

    let mut cam = Camera2D::from_display_rect(cam_rect);
    cam.zoom.y = -cam.zoom.y;
    set_camera(&cam);

    // Cyber grid
    let grid = 80.0;
    let start_x = ((cam_rect.x / grid).floor() * grid).max(-ARENA_RADIUS);
    let end_x = (cam_rect.x + vw).min(ARENA_RADIUS);
    let start_y = ((cam_rect.y / grid).floor() * grid).max(-ARENA_RADIUS);
    let end_y = (cam_rect.y + vh).min(ARENA_RADIUS);
    let grid_col = Color::from_rgba(5, 217, 232, 18);

    let mut gx = start_x;
    while gx <= end_x {
        draw_line(gx, start_y, gx, end_y, 1.0, grid_col);
        gx += grid;
    }
    let mut gy = start_y;
    while gy <= end_y {
        draw_line(start_x, gy, end_x, gy, 1.0, grid_col);
        gy += grid;
    }

    // Sector landmark rings & arena outer boundary
    let ring_col = Color::from_rgba(5, 217, 232, 30);
    draw_circle_lines(0.0, 0.0, 950.0, 1.5, ring_col);
    draw_circle_lines(0.0, 0.0, 1900.0, 1.5, ring_col);
    draw_circle_lines(
        0.0,
        0.0,
        ARENA_RADIUS,
        14.0,
        Color::from_rgba(255, 42, 109, 55),
    );
    draw_circle_lines(
        0.0,
        0.0,
        ARENA_RADIUS,
        2.5,
        Color::from_rgba(255, 42, 109, 255),
    );

    // Overload Beacon capture zone
    if game.beacon.active {
        let b = game.beacon;
        if b.pos.x + b.radius >= vmin_x
            && b.pos.x - b.radius <= vmax_x
            && b.pos.y + b.radius >= vmin_y
            && b.pos.y - b.radius <= vmax_y
        {
            let gold = Color::from_rgba(249, 200, 14, 255);
            draw_circle(
                b.pos.x,
                b.pos.y,
                b.radius,
                Color::from_rgba(249, 200, 14, 22),
            );
            draw_circle_lines(
                b.pos.x,
                b.pos.y,
                b.radius,
                2.0,
                Color::from_rgba(249, 200, 14, 150),
            );
            let prog = (b.progress / b.duration).clamp(0.0, 1.0);
            if prog > 0.0 {
                let segs = 28;
                let max_a = prog * std::f32::consts::TAU;
                let start_a = -std::f32::consts::FRAC_PI_2;
                let r_arc = b.radius - 4.0;
                for seg in 0..segs {
                    let a0 = start_a + (seg as f32 / segs as f32) * max_a;
                    let a1 = start_a + ((seg + 1) as f32 / segs as f32) * max_a;
                    let p0 = b.pos + Vec2::from_angle(a0) * r_arc;
                    let p1 = b.pos + Vec2::from_angle(a1) * r_arc;
                    draw_line(p0.x, p0.y, p1.x, p1.y, 4.5, gold);
                }
            }
            draw_poly_lines(
                b.pos.x,
                b.pos.y,
                4,
                16.0,
                (b.ttl * 2.2).to_degrees(),
                2.5,
                gold,
            );
            draw_centered_text("PRZEKAZNIK", b.pos.x, b.pos.y - b.radius - 8.0, 14.0, gold);
        }
    }

    // Orbital Hazard Strike telegraphs
    for h in &game.hazards {
        if h.pos.x + h.radius < vmin_x
            || h.pos.x - h.radius > vmax_x
            || h.pos.y + h.radius < vmin_y
            || h.pos.y - h.radius > vmax_y
        {
            continue;
        }
        let p = 1.0 - (h.timer / h.max_timer).clamp(0.0, 1.0);
        let fill_a = ((0.07 + p * 0.14) * 255.0) as u8;
        draw_circle(
            h.pos.x,
            h.pos.y,
            h.radius,
            Color::from_rgba(255, 58, 92, fill_a),
        );
        draw_circle_lines(
            h.pos.x,
            h.pos.y,
            h.radius,
            2.0,
            Color::from_rgba(255, 58, 92, 200),
        );
        draw_circle_lines(
            h.pos.x,
            h.pos.y,
            (h.radius * p).max(2.0),
            2.5,
            Color::from_rgba(255, 58, 92, 245),
        );
        let cross = Color::from_rgba(255, 255, 255, 190);
        draw_line(h.pos.x - 10.0, h.pos.y, h.pos.x + 10.0, h.pos.y, 1.5, cross);
        draw_line(h.pos.x, h.pos.y - 10.0, h.pos.x, h.pos.y + 10.0, 1.5, cross);
    }

    // Expanding rings
    for r in &game.effects.rings {
        if r.pos.x + r.radius < vmin_x
            || r.pos.x - r.radius > vmax_x
            || r.pos.y + r.radius < vmin_y
            || r.pos.y - r.radius > vmax_y
        {
            continue;
        }
        let mut c = r.color;
        c.a = (r.life / r.max_life).clamp(0.0, 1.0);
        draw_circle_lines(r.pos.x, r.pos.y, r.radius, 3.0, c);
    }

    // XP & Shard gems
    for g in &game.gems {
        if g.pos.x < vmin_x || g.pos.x > vmax_x || g.pos.y < vmin_y || g.pos.y > vmax_y {
            continue;
        }
        if g.shard > 0 {
            draw_circle(g.pos.x, g.pos.y, 5.5, Color::from_rgba(249, 200, 14, 255));
        } else {
            draw_circle(g.pos.x, g.pos.y, 3.5, Color::from_rgba(5, 217, 232, 255));
        }
    }

    // Player bullets
    for b in &game.bullets {
        if b.pos.x < vmin_x || b.pos.x > vmax_x || b.pos.y < vmin_y || b.pos.y > vmax_y {
            continue;
        }
        let tail = b.pos - b.vel * 0.028;
        draw_line(
            b.pos.x,
            b.pos.y,
            tail.x,
            tail.y,
            7.0,
            Color::from_rgba(5, 217, 232, 85),
        );
        draw_line(b.pos.x, b.pos.y, tail.x, tail.y, 2.5, WHITE);
    }

    // Tesla lightning arcs
    let arc_t = get_time() as f32;
    for (idx, a) in game.effects.arcs.iter().enumerate() {
        let phase = arc_t * 60.0 + idx as f32 * 3.1;
        let mid = (a.a + a.b) * 0.5 + vec2(phase.sin() * 10.0, (phase * 1.37).cos() * 10.0);
        let col = Color::from_rgba(125, 249, 255, 255);
        draw_line(a.a.x, a.a.y, mid.x, mid.y, 2.5, col);
        draw_line(mid.x, mid.y, a.b.x, a.b.y, 2.5, col);
    }

    // Enemy bullets
    for eb in &game.e_bullets {
        if eb.pos.x < vmin_x || eb.pos.x > vmax_x || eb.pos.y < vmin_y || eb.pos.y > vmax_y {
            continue;
        }
        draw_circle(
            eb.pos.x,
            eb.pos.y,
            eb.radius * 1.9,
            Color::from_rgba(255, 58, 92, 76),
        );
        draw_circle(
            eb.pos.x,
            eb.pos.y,
            eb.radius,
            Color::from_rgba(255, 58, 92, 255),
        );
        draw_circle(eb.pos.x, eb.pos.y, eb.radius * 0.45, WHITE);
    }

    // Enemies (7 archetypes)
    for e in &game.enemies {
        if !e.alive {
            continue;
        }
        if e.pos.x + e.radius + 20.0 < vmin_x
            || e.pos.x - e.radius - 20.0 > vmax_x
            || e.pos.y + e.radius + 20.0 < vmin_y
            || e.pos.y - e.radius - 20.0 > vmax_y
        {
            continue;
        }

        if e.kind == EnemyKind::Sniper && e.shoot_cd > 0.0 && e.shoot_cd < 0.45 {
            let la = ((0.35 + (0.45 - e.shoot_cd) * 1.4).clamp(0.0, 1.0) * 255.0) as u8;
            draw_line(
                e.pos.x,
                e.pos.y,
                game.player.pos.x,
                game.player.pos.y,
                1.5,
                Color::from_rgba(255, 58, 92, la),
            );
        }
        if e.kind == EnemyKind::Vortex {
            let vr = 44.0 + (e.timer * 5.0).sin() * 6.0;
            draw_circle_lines(
                e.pos.x,
                e.pos.y,
                vr,
                1.5,
                Color::from_rgba(125, 110, 255, 72),
            );
        }

        let enraged_boss = e.kind == EnemyKind::Boss && e.hp < e.max_hp * 0.5;
        let base_col = if e.flash > 0.0 {
            WHITE
        } else {
            e.kind.base_color(enraged_boss)
        };
        if e.elite {
            draw_circle_lines(
                e.pos.x,
                e.pos.y,
                e.radius + 5.0,
                1.5,
                Color::from_rgba(255, 255, 255, 165),
            );
        }
        let mut glow_col = base_col;
        glow_col.a = 0.28;
        let draw_r = if e.flash > 0.0 {
            e.radius * 1.12
        } else {
            e.radius
        };
        let deg = e.angle.to_degrees();
        let sides = e.kind.polygon_sides();
        draw_poly_lines(e.pos.x, e.pos.y, sides, draw_r, deg, 7.0, glow_col);
        draw_poly_lines(e.pos.x, e.pos.y, sides, draw_r, deg, 2.2, base_col);

        if (e.kind == EnemyKind::Tank
            || e.kind == EnemyKind::Boss
            || e.kind == EnemyKind::Vortex
            || e.elite)
            && e.hp < e.max_hp
        {
            let bw = e.radius * 2.0;
            let ratio = (e.hp / e.max_hp).clamp(0.0, 1.0);
            draw_rectangle(
                e.pos.x - e.radius,
                e.pos.y - e.radius - 10.0,
                bw,
                4.0,
                Color::from_rgba(255, 255, 255, 50),
            );
            draw_rectangle(
                e.pos.x - e.radius,
                e.pos.y - e.radius - 10.0,
                bw * ratio,
                4.0,
                base_col,
            );
        }
    }

    // Orbital saws
    if game.player.orbit_count > 0 {
        let col = Color::from_rgba(5, 255, 161, 255);
        for i in 0..game.player.orbit_count {
            let a = game.player.orbit_angle
                + (i as f32 * std::f32::consts::TAU) / game.player.orbit_count as f32;
            let bpos = game.player.pos + Vec2::from_angle(a) * 74.0;
            draw_circle_lines(bpos.x, bpos.y, 8.5, 2.5, col);
        }
    }

    // Phantom Wraith Plasma Mines
    for m in &game.mines {
        if m.pos.x + 28.0 < vmin_x
            || m.pos.x - 28.0 > vmax_x
            || m.pos.y + 28.0 < vmin_y
            || m.pos.y - 28.0 > vmax_y
        {
            continue;
        }
        let pulse = 18.0 + (m.life * 8.0).sin() * 3.0;
        draw_circle_lines(
            m.pos.x,
            m.pos.y,
            pulse,
            1.5,
            Color::from_rgba(184, 41, 234, 110),
        );
        draw_poly_lines(
            m.pos.x,
            m.pos.y,
            6,
            9.0,
            (m.life * 2.5).to_degrees(),
            2.2,
            Color::from_rgba(184, 41, 234, 255),
        );
        draw_circle(m.pos.x, m.pos.y, 3.5, Color::from_rgba(255, 42, 109, 255));
    }

    // Player ship (stack-allocated hull geometry)
    let p = game.player.pos;
    let ship = game.selected_ship();
    if game.player.max_shield > 0.0 && game.player.shield > 0.5 {
        let sa = ((0.28 + 0.42 * (game.player.shield / game.player.max_shield)).clamp(0.0, 1.0)
            * 255.0) as u8;
        draw_circle_lines(p.x, p.y, 25.0, 2.2, Color::from_rgba(125, 249, 255, sa));
    }
    let ship_col = if game.player.dash_timer > 0.0 {
        WHITE
    } else if game.player.overdrive_timer > 0.0 {
        Color::from_rgba(249, 200, 14, 255)
    } else {
        ship.color()
    };
    let glow = if game.player.overdrive_timer > 0.0 {
        Color::from_rgba(249, 200, 14, 115)
    } else {
        ship.color_a(85)
    };
    let (pts, n_pts) = ship_hull_vertices(ship.id, p, game.player.aim_angle);
    for i in 0..n_pts {
        let a = pts[i];
        let b = pts[(i + 1) % n_pts];
        draw_line(a.x, a.y, b.x, b.y, 7.5, glow);
        draw_line(a.x, a.y, b.x, b.y, 2.5, ship_col);
    }

    // Off-screen Overload Beacon indicator
    if game.beacon.active {
        let bdiff = game.beacon.pos - p;
        if bdiff.x.abs() > vw * 0.44 || bdiff.y.abs() > vh * 0.44 {
            let ind = p + bdiff.normalize() * 56.0;
            let deg = bdiff.y.atan2(bdiff.x).to_degrees();
            draw_poly_lines(
                ind.x,
                ind.y,
                4,
                6.5,
                deg,
                2.2,
                Color::from_rgba(249, 200, 14, 255),
            );
        }
    }

    // Off-screen enemy radar dots
    for (idx, e) in game.enemies.iter().enumerate() {
        if !e.alive {
            continue;
        }
        if e.kind != EnemyKind::Boss && !e.elite && (idx & 3) != 0 {
            continue;
        }
        let diff = e.pos - p;
        if diff.x.abs() > vw * 0.48 || diff.y.abs() > vh * 0.48 {
            let ind = p + diff.normalize() * 46.0;
            let c = if e.kind == EnemyKind::Boss {
                Color::from_rgba(184, 41, 234, 255)
            } else if e.elite {
                WHITE
            } else {
                Color::from_rgba(255, 42, 109, 170)
            };
            let r = if e.kind == EnemyKind::Boss {
                4.5
            } else if e.elite {
                3.5
            } else {
                2.5
            };
            draw_circle(ind.x, ind.y, r, c);
        }
    }

    // Particles
    for pt in &game.effects.particles {
        if pt.pos.x < vmin_x || pt.pos.x > vmax_x || pt.pos.y < vmin_y || pt.pos.y > vmax_y {
            continue;
        }
        let mut c = pt.color;
        c.a = (pt.life / pt.max_life).clamp(0.0, 1.0);
        draw_rectangle(
            pt.pos.x - pt.size * 0.5,
            pt.pos.y - pt.size * 0.5,
            pt.size,
            pt.size,
            c,
        );
    }

    set_default_camera();

    // Floating damage & +XP / +SHARD popups rendered at native high-DPI ui_scale
    let s = vp.scale;
    for t in &game.effects.texts {
        if t.pos.x < vmin_x || t.pos.x > vmax_x || t.pos.y < vmin_y || t.pos.y > vmax_y {
            continue;
        }
        let scr_x = (t.pos.x - cam_rect.x) * s;
        let scr_y = (t.pos.y - cam_rect.y) * s;
        let alpha = (t.life * 2.0).clamp(0.0, 1.0);
        let mut c = t.color;
        c.a = alpha;
        let f_sz = 15.5 * s;
        let shadow = Color::from_rgba(4, 6, 11, (alpha * 210.0) as u8);
        draw_centered_text(
            t.val.as_str(),
            scr_x + 1.2 * s,
            scr_y + 1.2 * s,
            f_sz,
            shadow,
        );
        draw_centered_text(t.val.as_str(), scr_x, scr_y, f_sz, c);
    }
}
