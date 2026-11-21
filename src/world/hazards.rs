//! Orbital Hazard Strike telegraphs and detonation logic.

use super::arena::clamp_to_arena;
use crate::game::config::MAX_HAZARDS;
use crate::progression::{DifficultySpec, WaveEvent};
use crate::rng::GameRng;
use macroquad::math::{vec2, Vec2};

#[derive(Clone, Copy, Debug)]
pub struct HazardZone {
    pub pos: Vec2,
    pub radius: f32,
    pub timer: f32,
    pub max_timer: f32,
    pub dmg: f32,
}

pub fn spawn_hazard(
    hazards: &mut Vec<HazardZone>,
    pos: Vec2,
    radius: f32,
    max_timer: f32,
    dmg: f32,
) {
    if hazards.len() >= MAX_HAZARDS {
        return;
    }
    let clamped = clamp_to_arena(pos, radius + 20.0);
    let safe_timer = max_timer.max(0.1);
    hazards.push(HazardZone {
        pos: clamped,
        radius,
        timer: safe_timer,
        max_timer: safe_timer,
        dmg,
    });
}

/// Ticks hazard spawn timer and spawns telegraphed orbital strikes when due.
#[allow(clippy::too_many_arguments)]
pub fn tick_hazard_spawner(
    hazards: &mut Vec<HazardZone>,
    spawn_timer: &mut f32,
    dt: f32,
    wave: u32,
    wave_event: WaveEvent,
    diff: DifficultySpec,
    player_pos: Vec2,
    player_vel: Vec2,
    rng: &mut GameRng,
) {
    let min_wave = if diff.id == 2 { 2 } else { 3 };
    if wave < min_wave && wave_event != WaveEvent::Orbital {
        return;
    }

    *spawn_timer -= dt;
    if *spawn_timer > 0.0 {
        return;
    }

    let is_orbital = wave_event == WaveEvent::Orbital;
    let h_div = 1.0 + diff.id as f32 * 0.16;
    *spawn_timer = if is_orbital {
        ((1.05 - wave as f32 * 0.04) / h_div).max(0.36)
    } else {
        ((3.4 - wave as f32 * 0.14) / h_div).max(1.0)
    };
    let count = if is_orbital {
        if wave >= 7 || diff.id == 2 {
            3
        } else {
            2
        }
    } else {
        1
    };

    for h_idx in 0..count {
        let lead = if h_idx == 0 {
            0.45
        } else {
            rng.range_f32(0.1, 0.85)
        };
        let hpos = player_pos
            + player_vel * lead
            + vec2(rng.range_f32(-140.0, 140.0), rng.range_f32(-140.0, 140.0));
        let hrad = if is_orbital {
            rng.range_f32(66.0, 86.0)
        } else {
            rng.range_f32(58.0, 76.0)
        };
        let warn_t = (1.38 - wave as f32 * 0.025 - diff.id as f32 * 0.08).max(0.84);
        let hdmg = (22.0 + wave as f32 * 2.8) * diff.dmg_mult;
        spawn_hazard(hazards, hpos, hrad, warn_t, hdmg);
    }
}
