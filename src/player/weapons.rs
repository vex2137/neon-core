//! Player aiming resolution and primary weapon salvo firing.

use super::stats::PlayerState;
use crate::audio::SoundBank;
use crate::combat::Bullet;
use crate::game::config::MAX_BULLETS;
use macroquad::math::Vec2;

/// Updates `player.aim_angle` from manual aim, auto-aim target, or movement velocity.
pub fn update_aim(
    player: &mut PlayerState,
    manual_aim_dir: Option<Vec2>,
    nearest_enemy_pos: Option<Vec2>,
    auto_aim: bool,
) {
    if let Some(dir) = manual_aim_dir {
        if dir.length_squared() > 0.0001 {
            player.aim_angle = dir.y.atan2(dir.x);
            return;
        }
    }

    if auto_aim {
        if let Some(target) = nearest_enemy_pos {
            let d = target - player.pos;
            player.aim_angle = d.y.atan2(d.x);
            return;
        }
    }

    if player.vel.length_squared() > 400.0 {
        player.aim_angle = player.vel.y.atan2(player.vel.x);
    }
}

/// Ticks weapon fire cooldown and emits primary weapon salvos.
#[allow(clippy::too_many_arguments)]
pub fn update_primary_weapon(
    player: &mut PlayerState,
    bullets: &mut Vec<Bullet>,
    sounds: &mut SoundBank,
    sfx_gain: f32,
    auto_aim: bool,
    auto_fire: bool,
    fire_held: bool,
    manual_aim: bool,
    has_target: bool,
    overdrive_lvl: u32,
    dt: f32,
) {
    sounds.tick_cooldowns(dt);
    player.fire_cd -= dt;
    let should_fire = if auto_fire {
        manual_aim || !auto_aim || has_target || fire_held
    } else {
        fire_held
    };
    if player.fire_cd > 0.0 || !should_fire {
        return;
    }

    player.fire_cd = if player.overdrive_timer > 0.0 {
        player.fire_interval * 0.68
    } else {
        player.fire_interval
    };
    sounds.play_shoot_throttled(sfx_gain);

    let spread = player.bullet_spread;
    let shot_dmg = if player.overdrive_timer > 0.0 {
        player.bullet_dmg * (1.0 + overdrive_lvl as f32 * 0.12)
    } else {
        player.bullet_dmg
    };

    for i in 0..player.bullet_count {
        if bullets.len() >= MAX_BULLETS {
            break;
        }
        let offset = (i as f32 - (player.bullet_count - 1) as f32 * 0.5) * spread;
        let a = player.aim_angle + offset;
        let dir = Vec2::from_angle(a);
        bullets.push(Bullet {
            pos: player.pos + dir * 18.0,
            vel: dir * 760.0,
            dmg: shot_dmg,
            life: 1.15,
            pierce: player.bullet_pierce,
        });
    }
}
