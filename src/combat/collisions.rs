//! Spatial collision helpers for enemy bullets, hazards, and XP/shard gems.

use super::projectiles::EnemyBullet;
use crate::audio::SoundBank;
use crate::effects::EffectsPool;
use crate::game::config::ARENA_RADIUS;
use crate::progression::{combo_xp_multiplier, scaled_shard_pickup, Gem};
use crate::rng::GameRng;
use crate::world::HazardZone;
use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};

#[derive(Clone, Copy, Debug, Default)]
pub struct IncomingPlayerHits {
    pub damage: f32,
    pub shake: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GemPickupTotals {
    pub xp_gained: f32,
    pub shards_gained: u32,
}

/// Advances enemy bullets, culls out-of-bounds/expired shots, and collects hits on the player.
pub fn update_enemy_bullets(
    e_bullets: &mut Vec<EnemyBullet>,
    player_pos: Vec2,
    player_dashing: bool,
    dt: f32,
    effects: &mut EffectsPool,
    rng: &mut GameRng,
) -> IncomingPlayerHits {
    let arena_r2 = ARENA_RADIUS * ARENA_RADIUS;
    let mut hits = IncomingPlayerHits::default();
    let mut ebi = e_bullets.len();
    while ebi > 0 {
        ebi -= 1;
        let vel = e_bullets[ebi].vel;
        e_bullets[ebi].pos += vel * dt;
        e_bullets[ebi].life -= dt;
        let pos = e_bullets[ebi].pos;
        if e_bullets[ebi].life <= 0.0 || pos.length_squared() > arena_r2 {
            e_bullets.swap_remove(ebi);
            continue;
        }
        if !player_dashing {
            let hit_r = e_bullets[ebi].radius + 10.5;
            if pos.distance_squared(player_pos) < hit_r * hit_r {
                let dmg = e_bullets[ebi].dmg;
                e_bullets.swap_remove(ebi);
                effects.spawn_particle(
                    player_pos,
                    vec2(rng.range_f32(-90.0, 90.0), rng.range_f32(-90.0, 90.0)),
                    0.22,
                    4.0,
                    Color::from_rgba(255, 58, 92, 255),
                );
                hits.damage += dmg;
                hits.shake = hits.shake.max(7.0);
            }
        }
    }
    hits
}

/// Advances orbital hazard countdowns, spawns detonation VFX, and collects hits on the player.
#[allow(clippy::too_many_arguments)]
pub fn update_hazards(
    hazards: &mut Vec<HazardZone>,
    player_pos: Vec2,
    player_dashing: bool,
    dt: f32,
    effects: &mut EffectsPool,
    sounds: &SoundBank,
    sfx_gain: f32,
    rng: &mut GameRng,
) -> IncomingPlayerHits {
    let mut hits = IncomingPlayerHits::default();
    let mut hi = hazards.len();
    while hi > 0 {
        hi -= 1;
        hazards[hi].timer -= dt;
        if hazards[hi].timer <= 0.0 {
            let h = hazards.swap_remove(hi);
            let crimson = Color::from_rgba(255, 58, 92, 255);
            effects.spawn_ring(h.pos, h.radius * 1.15, crimson, 0.34);
            sounds.play(&sounds.boom, sfx_gain, 0.55);
            for _ in 0..10 {
                let a = rng.range_f32(0.0, std::f32::consts::TAU);
                let spd = rng.range_f32(60.0, 210.0);
                effects.spawn_particle(
                    h.pos,
                    Vec2::from_angle(a) * spd,
                    rng.range_f32(0.2, 0.42),
                    3.5,
                    crimson,
                );
            }
            if !player_dashing {
                let hr = h.radius + 8.0;
                if player_pos.distance_squared(h.pos) <= hr * hr {
                    hits.damage += h.dmg;
                    hits.shake = hits.shake.max(10.0);
                }
            }
        }
    }
    hits
}

/// Pulls gems within `magnet_radius` toward the player and collects pickups.
pub fn update_gems(
    gems: &mut Vec<Gem>,
    player_pos: Vec2,
    magnet_radius: f32,
    greed_mult: f32,
    combo: u32,
    dt: f32,
) -> GemPickupTotals {
    let mag_r2 = magnet_radius * magnet_radius;
    let combo_mult = combo_xp_multiplier(combo);
    let mut totals = GemPickupTotals::default();

    let mut gi = gems.len();
    while gi > 0 {
        gi -= 1;
        let to_p = player_pos - gems[gi].pos;
        let dist_sq = to_p.length_squared();
        if dist_sq < mag_r2 && dist_sq > 0.01 {
            gems[gi].pos += (to_p / dist_sq.sqrt()) * 510.0 * dt;
        }
        if dist_sq < 24.0 * 24.0 {
            let g = gems.swap_remove(gi);
            totals.xp_gained += g.xp * greed_mult * combo_mult;
            if g.shard > 0 {
                totals.shards_gained = totals
                    .shards_gained
                    .saturating_add(scaled_shard_pickup(g.shard, greed_mult));
            }
        }
    }
    totals
}
