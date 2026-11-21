//! Enemy archetype AI movement, projectile firing patterns, and boss phases.

use super::types::{Enemy, EnemyKind};
use crate::audio::SoundBank;
use crate::combat::EnemyBullet;
use crate::effects::EffectsPool;
use crate::game::config::MAX_EBULLETS;
use crate::progression::DifficultySpec;
use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};

#[derive(Clone, Copy, Debug)]
pub struct EnemyAiOutcome {
    /// Total contact/explosion damage dealt to the player this step (0.0 if none or dashing).
    pub player_damage: f32,
    pub player_shake: f32,
    /// Net pull vector applied to `player_pos` by Vortex elites.
    pub player_pull: Vec2,
    /// True if any enemy became dead (e.g., Shrapnel Bomber self-detonated).
    pub any_died: bool,
}

fn push_enemy_bullet(
    e_bullets: &mut Vec<EnemyBullet>,
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    radius: f32,
) {
    if e_bullets.len() >= MAX_EBULLETS {
        return;
    }
    e_bullets.push(EnemyBullet {
        pos,
        vel,
        dmg,
        life: 3.6,
        radius,
    });
}

/// Updates all living enemies: movement, archetype attacks, boss phases, vortex pull, and contact damage.
#[allow(clippy::too_many_arguments)]
pub fn update_enemies_ai(
    enemies: &mut [Enemy],
    e_bullets: &mut Vec<EnemyBullet>,
    effects: &mut EffectsPool,
    sounds: &SoundBank,
    sound_enabled: bool,
    player_pos: Vec2,
    player_dashing: bool,
    wave: u32,
    diff: DifficultySpec,
    dt: f32,
) -> EnemyAiOutcome {
    let e_dmg_scale = (1.0 + wave.saturating_sub(1) as f32 * 0.14) * diff.dmg_mult;
    let mut outcome = EnemyAiOutcome {
        player_damage: 0.0,
        player_shake: 0.0,
        player_pull: Vec2::ZERO,
        any_died: false,
    };

    for (i, enemy) in enemies.iter_mut().enumerate() {
        if !enemy.alive {
            continue;
        }
        if enemy.flash > 0.0 {
            enemy.flash = (enemy.flash - dt).max(0.0);
        }

        let epos = enemy.pos;
        let kind = enemy.kind;
        let elite = enemy.elite;
        let erad = enemy.radius;
        let to_player = player_pos - epos;
        let dist_sq = to_player.length_squared();
        let dist = if dist_sq > 0.000_001 {
            dist_sq.sqrt()
        } else {
            0.001
        };
        let dir = to_player / dist;

        enemy.angle += enemy.spin * dt;
        enemy.timer += dt;
        let e_angle = enemy.angle;
        let e_speed = enemy.speed;

        match kind {
            EnemyKind::Dasher => {
                let mult = if (enemy.timer % 2.1) > 1.55 {
                    3.25
                } else {
                    0.65
                };
                enemy.pos += dir * e_speed * mult * dt;
            }
            EnemyKind::Sniper => {
                let strafe_dir = if (i & 1) == 0 { 1.0 } else { -1.0 };
                let radial = if dist > 360.0 {
                    1.0
                } else if dist < 245.0 {
                    -0.85
                } else {
                    0.05
                };
                let mv = vec2(
                    dir.x * radial - dir.y * 0.72 * strafe_dir,
                    dir.y * radial + dir.x * 0.72 * strafe_dir,
                );
                enemy.pos += mv * e_speed * dt;

                enemy.shoot_cd -= dt;
                if enemy.shoot_cd <= 0.0 && dist < 680.0 {
                    enemy.shoot_cd = (2.35 - wave as f32 * 0.06).max(1.35);
                    let base_a = to_player.y.atan2(to_player.x);
                    let shot_spd = 460.0 + (wave as f32 * 14.0).min(180.0);
                    let shot_dmg = (if elite { 18.0 } else { 13.0 }) * e_dmg_scale;
                    if elite {
                        for s in -1..=1 {
                            let a = base_a + s as f32 * 0.16;
                            push_enemy_bullet(
                                e_bullets,
                                epos,
                                Vec2::from_angle(a) * shot_spd,
                                shot_dmg,
                                5.5,
                            );
                        }
                    } else {
                        push_enemy_bullet(
                            e_bullets,
                            epos,
                            Vec2::from_angle(base_a) * shot_spd,
                            shot_dmg,
                            5.0,
                        );
                    }
                    sounds.play(&sounds.shoot, sound_enabled, 0.35);
                }
            }
            EnemyKind::Bomber => {
                enemy.pos += dir * e_speed * dt;
                if dist < 68.0 {
                    enemy.alive = false;
                    outcome.any_died = true;
                    effects.spawn_ring(epos, 115.0, Color::from_rgba(170, 255, 45, 255), 0.32);
                    sounds.play(&sounds.boom, sound_enabled, 0.75);
                    let pellets = if elite { 10 } else { 7 };
                    let b_spd = 250.0 + (wave as f32 * 8.0).min(120.0);
                    let b_dmg = 13.0 * e_dmg_scale;
                    for p in 0..pellets {
                        let a = e_angle + (p as f32 * std::f32::consts::TAU) / pellets as f32;
                        push_enemy_bullet(e_bullets, epos, Vec2::from_angle(a) * b_spd, b_dmg, 5.0);
                    }
                    if !player_dashing && dist < 82.0 {
                        outcome.player_damage += 20.0 * e_dmg_scale;
                        outcome.player_shake = outcome.player_shake.max(9.0);
                    }
                    continue;
                }
            }
            EnemyKind::Vortex => {
                let strafe_dir = if (i & 1) == 0 { 1.0 } else { -1.0 };
                let radial = if dist > 290.0 {
                    0.9
                } else if dist < 195.0 {
                    -0.6
                } else {
                    0.0
                };
                let mv = vec2(
                    dir.x * radial - dir.y * 0.85 * strafe_dir,
                    dir.y * radial + dir.x * 0.85 * strafe_dir,
                );
                enemy.pos += mv * e_speed * dt;

                if dist < 270.0 && !player_dashing {
                    let pull = (1.0 - dist / 270.0) * (if elite { 115.0 } else { 80.0 }) * dt;
                    outcome.player_pull -= dir * pull;
                }
                enemy.shoot_cd -= dt;
                if enemy.shoot_cd <= 0.0 && dist < 640.0 {
                    enemy.shoot_cd = (2.5 - wave as f32 * 0.06).max(1.45);
                    let count = if elite { 6 } else { 4 };
                    let b_spd = 220.0 + (wave as f32 * 8.0).min(110.0);
                    let b_dmg = 12.0 * e_dmg_scale;
                    for s in 0..count {
                        let a = e_angle + (s as f32 * std::f32::consts::TAU) / count as f32;
                        push_enemy_bullet(e_bullets, epos, Vec2::from_angle(a) * b_spd, b_dmg, 5.2);
                    }
                }
            }
            EnemyKind::Boss => {
                let enraged = enemy.hp < enemy.max_hp * 0.5;
                let step = e_speed * if enraged { 1.32 } else { 1.0 } * dt;
                enemy.pos += dir * step;

                enemy.shoot_cd -= dt;
                if enemy.shoot_cd <= 0.0 && dist < 760.0 {
                    enemy.shoot_cd = if enraged {
                        (1.35 - wave as f32 * 0.025).max(0.85)
                    } else {
                        (2.0 - wave as f32 * 0.03).max(1.25)
                    };
                    let ring_cnt = ((if enraged { 12 } else { 8 }) + (wave / 3)).min(16);
                    let b_spd =
                        (if enraged { 275.0 } else { 225.0 }) + (wave as f32 * 9.0).min(120.0);
                    let b_dmg = 15.0 * e_dmg_scale;
                    for s in 0..ring_cnt {
                        let a = e_angle + (s as f32 * std::f32::consts::TAU) / ring_cnt as f32;
                        push_enemy_bullet(e_bullets, epos, Vec2::from_angle(a) * b_spd, b_dmg, 6.0);
                    }
                    if enraged {
                        let aim_a = to_player.y.atan2(to_player.x);
                        push_enemy_bullet(
                            e_bullets,
                            epos,
                            Vec2::from_angle(aim_a) * (b_spd + 140.0),
                            b_dmg * 1.15,
                            6.5,
                        );
                    }
                    let rcol = if enraged {
                        Color::from_rgba(255, 58, 92, 255)
                    } else {
                        Color::from_rgba(184, 41, 234, 255)
                    };
                    effects.spawn_ring(epos, erad * 2.1, rcol, 0.28);
                }
            }
            EnemyKind::Swarmer | EnemyKind::Tank => {
                enemy.pos += dir * e_speed * dt;
            }
        }

        let contact_r = erad + 14.0;
        if !player_dashing && dist_sq < contact_r * contact_r {
            let base_dps = match kind {
                EnemyKind::Boss => 78.0,
                EnemyKind::Tank => 52.0,
                _ => 42.0,
            };
            let dps = base_dps * if elite { 1.3 } else { 1.0 };
            outcome.player_damage += dps * dt;
            outcome.player_shake = outcome.player_shake.max(5.0);
        }
    }

    outcome
}
