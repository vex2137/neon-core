//! Arena boundary helpers and background parallax starfield generation.

use crate::game::config::ARENA_RADIUS;
use crate::rng::GameRng;
use macroquad::math::{vec2, Vec2};

#[derive(Clone, Copy, Debug)]
pub struct Star {
    pub pos: Vec2,
    pub depth: f32,
    pub size: f32,
}

/// Clamps `pos` to stay within `ARENA_RADIUS - margin`. Non-finite coordinates reset to origin.
#[inline]
pub fn clamp_to_arena(pos: Vec2, margin: f32) -> Vec2 {
    if !pos.x.is_finite() || !pos.y.is_finite() {
        return Vec2::ZERO;
    }
    let limit = (ARENA_RADIUS - margin).max(1.0);
    let d2 = pos.length_squared();
    if d2 > limit * limit {
        (pos / d2.sqrt()) * limit
    } else {
        pos
    }
}

pub fn generate_starfield(rng: &mut GameRng, count: usize) -> Vec<Star> {
    let mut stars = Vec::with_capacity(count);
    for _ in 0..count {
        stars.push(Star {
            pos: vec2(
                rng.range_f32(-1600.0, 1600.0),
                rng.range_f32(-1600.0, 1600.0),
            ),
            depth: if rng.range_u32(0, 10) < 6 { 0.25 } else { 0.55 },
            size: rng.range_f32(1.5, 2.6),
        });
    }
    stars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_to_arena_enforces_boundary_and_rejects_nan() {
        let clamped = clamp_to_arena(vec2(5000.0, 0.0), 18.0);
        assert!((clamped.x - (ARENA_RADIUS - 18.0)).abs() < 1e-3);
        assert_eq!(clamp_to_arena(vec2(f32::NAN, 10.0), 18.0), Vec2::ZERO);
    }
}
