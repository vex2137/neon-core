//! Enemy wave scaling, archetype roll weights, and factory construction.

use super::types::{Enemy, EnemyKind};
use crate::game::config::{Viewport, ARENA_RADIUS};
use crate::progression::{DifficultySpec, WaveEvent};
use crate::rng::GameRng;
use macroquad::math::Vec2;

#[inline]
pub fn hp_scale_for_wave(wave: u32, diff: DifficultySpec) -> f32 {
    let linear = (wave.saturating_sub(1)) as f32 * 0.30;
    let endgame = if wave > 4 {
        ((wave - 4) as f32).powf(1.5) * 0.26
    } else {
        0.0
    };
    (1.0 + linear + endgame) * diff.hp_mult
}

#[inline]
pub fn speed_scale_for_wave(wave: u32, diff: DifficultySpec) -> f32 {
    (1.0 + ((wave.saturating_sub(1)) as f32 * 0.045).min(0.65)) * diff.spd_mult
}

#[inline]
pub fn elite_chance_for_wave(wave: u32, diff: DifficultySpec) -> f32 {
    if wave >= diff.elite_wave {
        (0.10 + (wave - diff.elite_wave) as f32 * 0.03 + diff.id as f32 * 0.06).min(0.45)
    } else {
        0.0
    }
}

/// Picks an enemy archetype based on active wave number and wave event.
pub fn roll_wave_enemy_kind(wave: u32, event: WaveEvent, rng: &mut GameRng) -> EnemyKind {
    let r = rng.next_f32();
    match event {
        WaveEvent::Sniper => {
            if r > 0.72 {
                if wave >= 5 {
                    EnemyKind::Vortex
                } else {
                    EnemyKind::Sniper
                }
            } else if r > 0.36 {
                EnemyKind::Sniper
            } else if r > 0.18 {
                EnemyKind::Dasher
            } else {
                EnemyKind::Swarmer
            }
        }
        WaveEvent::Bomber => {
            if r > 0.56 {
                EnemyKind::Bomber
            } else if r > 0.34 {
                EnemyKind::Dasher
            } else if r > 0.22 {
                EnemyKind::Tank
            } else {
                EnemyKind::Swarmer
            }
        }
        WaveEvent::Swarm => {
            if r > 0.75 {
                EnemyKind::Dasher
            } else if r > 0.65 && wave >= 4 {
                EnemyKind::Bomber
            } else {
                EnemyKind::Swarmer
            }
        }
        _ => {
            if wave >= 5 && r > 0.88 {
                EnemyKind::Vortex
            } else if wave >= 4 && r > 0.74 {
                EnemyKind::Bomber
            } else if wave >= 3 && r > 0.62 {
                EnemyKind::Sniper
            } else if wave >= 3 && r > 0.56 {
                EnemyKind::Tank
            } else if wave >= 2 && r > 0.38 {
                EnemyKind::Dasher
            } else {
                EnemyKind::Swarmer
            }
        }
    }
}

/// Computes a clamped spawn point just outside the player's visible virtual viewport.
pub fn compute_spawn_position(player_pos: Vec2, vp: Viewport, rng: &mut GameRng) -> Vec2 {
    let angle = rng.range_f32(0.0, std::f32::consts::TAU);
    let dist = (vp.vw.max(vp.vh) * 0.56 + 75.0).min(ARENA_RADIUS - 80.0);
    let mut pos = player_pos + Vec2::from_angle(angle) * dist;
    let limit = ARENA_RADIUS - 45.0;
    let len_sq = pos.length_squared();
    if len_sq > limit * limit && len_sq.is_finite() {
        pos = (pos / len_sq.sqrt()) * limit;
    }
    pos
}

/// Constructs an `Enemy` scaled for the current wave and difficulty.
pub fn create_enemy(
    kind: EnemyKind,
    pos: Vec2,
    wave: u32,
    diff: DifficultySpec,
    allow_elite: bool,
    rng: &mut GameRng,
) -> Enemy {
    let hp_scale = hp_scale_for_wave(wave, diff);
    let spd_scale = speed_scale_for_wave(wave, diff);
    let elite = allow_elite
        && kind != EnemyKind::Boss
        && rng.next_f32() < elite_chance_for_wave(wave, diff);

    let (mut max_hp, mut speed, mut radius, shoot_cd) = match kind {
        EnemyKind::Swarmer => (24.0 * hp_scale, 118.0 * spd_scale, 12.0, 1.5),
        EnemyKind::Dasher => (36.0 * hp_scale, 138.0 * spd_scale, 13.0, 1.5),
        EnemyKind::Tank => (135.0 * hp_scale, 74.0 * spd_scale, 22.0, 1.5),
        EnemyKind::Boss => (640.0 * hp_scale, 68.0 * spd_scale, 36.0, 1.4),
        EnemyKind::Sniper => (
            48.0 * hp_scale,
            112.0 * spd_scale,
            15.0,
            rng.range_f32(1.2, 2.0),
        ),
        EnemyKind::Bomber => (42.0 * hp_scale, 158.0 * spd_scale, 14.0, 1.5),
        EnemyKind::Vortex => (
            105.0 * hp_scale,
            92.0 * spd_scale,
            19.0,
            rng.range_f32(1.1, 1.8),
        ),
    };

    if elite {
        max_hp *= 2.2;
        speed *= 1.15;
        radius *= 1.26;
    }

    Enemy {
        alive: true,
        kind,
        elite,
        pos,
        hp: max_hp,
        max_hp,
        radius,
        speed,
        flash: 0.0,
        timer: rng.range_f32(0.0, 2.0),
        shoot_cd,
        angle: rng.range_f32(0.0, std::f32::consts::TAU),
        spin: rng.range_f32(-3.5, 3.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression::DIFFICULTIES;
    use macroquad::math::vec2;

    #[test]
    fn boss_never_rolls_elite_even_on_nightmare() {
        let mut rng = GameRng::seeded(7);
        for _ in 0..50 {
            let e = create_enemy(
                EnemyKind::Boss,
                Vec2::ZERO,
                20,
                DIFFICULTIES[2],
                true,
                &mut rng,
            );
            assert!(!e.elite);
            assert_eq!(e.kind, EnemyKind::Boss);
            assert!(e.max_hp > 640.0);
        }
    }

    #[test]
    fn spawn_position_always_stays_within_arena() {
        let vp = Viewport::from_dimensions(2400.0, 1080.0);
        let mut rng = GameRng::seeded(123);
        let corner = vec2(ARENA_RADIUS - 20.0, 0.0);
        for _ in 0..100 {
            let p = compute_spawn_position(corner, vp, &mut rng);
            assert!(p.length() <= ARENA_RADIUS - 44.0);
        }
    }
}
