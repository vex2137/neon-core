//! Player movement, exponential acceleration, dash locomotion, and thruster trails.

use super::stats::PlayerState;
use crate::effects::EffectsPool;
use crate::rng::GameRng;
use crate::world::clamp_to_arena;
use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};

/// Updates player velocity, position, dash timer/cooldown, arena bounds, and engine trails.
pub fn update_player_movement(
    player: &mut PlayerState,
    move_dir: Vec2,
    dt: f32,
    effects: &mut EffectsPool,
    rng: &mut GameRng,
) {
    if player.dash_cd > 0.0 {
        player.dash_cd = (player.dash_cd - dt).max(0.0);
    }

    let trail_col = if player.overdrive_timer > 0.0 {
        Color::from_rgba(249, 200, 14, 255)
    } else {
        Color::from_rgba(5, 217, 232, 255)
    };
    let active_move_speed = if player.overdrive_timer > 0.0 {
        player.move_speed * 1.3
    } else {
        player.move_speed
    };

    if player.dash_timer > 0.0 {
        player.dash_timer = (player.dash_timer - dt).max(0.0);
        player.vel = player.dash_dir * 860.0;
        effects.spawn_particle(
            player.pos,
            vec2(rng.range_f32(-45.0, 45.0), rng.range_f32(-45.0, 45.0)),
            0.22,
            5.0,
            trail_col,
        );
    } else {
        let clamped_mv = if move_dir.length_squared() > 1.0 {
            move_dir.normalize()
        } else {
            move_dir
        };
        let target_vel = clamped_mv * active_move_speed;
        let accel = 1.0 - (-18.0 * dt).exp();
        player.vel += (target_vel - player.vel) * accel;

        if player.vel.length_squared() > 35.0 * 35.0 {
            player.thrust_accum += dt * 45.0;
            while player.thrust_accum >= 1.0 {
                player.thrust_accum -= 1.0;
                let back = player.vel.y.atan2(player.vel.x)
                    + std::f32::consts::PI
                    + rng.range_f32(-0.25, 0.25);
                let dir = Vec2::from_angle(back);
                effects.spawn_particle(player.pos + dir * 12.0, dir * 120.0, 0.2, 3.0, trail_col);
            }
        }
    }

    player.pos = clamp_to_arena(player.pos + player.vel * dt, 18.0);
}
