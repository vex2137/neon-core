//! Experience gems, combo XP multiplier, and next-level threshold scaling.

use macroquad::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Gem {
    pub pos: Vec2,
    pub xp: f32,
    pub shard: u32,
}

/// Combo XP multiplier: `+2.5%` per kill in chain, capped at `2.0x`.
#[inline]
pub fn combo_xp_multiplier(combo: u32) -> f32 {
    1.0 + (combo as f32 * 0.025).min(1.0)
}

/// Computes next level XP requirement from current requirement.
#[inline]
pub fn next_level_xp(current_next_xp: f32) -> f32 {
    let base = if current_next_xp.is_finite() && current_next_xp > 0.0 {
        current_next_xp
    } else {
        25.0
    };
    (base * 1.32 + 12.0).floor()
}

/// Computes shards awarded from a shard gem with greed/difficulty multiplier.
#[inline]
pub fn scaled_shard_pickup(base_shard: u32, greed_mult: f32) -> u32 {
    if base_shard == 0 {
        return 0;
    }
    let mult = if greed_mult.is_finite() && greed_mult > 0.0 {
        greed_mult
    } else {
        1.0
    };
    ((base_shard as f32 * mult).round() as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_multiplier_scales_and_caps_at_2x() {
        assert!((combo_xp_multiplier(0) - 1.0).abs() < 1e-5);
        assert!((combo_xp_multiplier(20) - 1.5).abs() < 1e-5);
        assert!((combo_xp_multiplier(40) - 2.0).abs() < 1e-5);
        assert!((combo_xp_multiplier(500) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn next_level_xp_follows_exact_curve() {
        assert_eq!(next_level_xp(25.0), 45.0);
        assert_eq!(next_level_xp(45.0), 71.0);
        assert_eq!(next_level_xp(f32::NAN), 45.0);
    }

    #[test]
    fn scaled_shard_pickup_preserves_minimum_one_for_nonzero() {
        assert_eq!(scaled_shard_pickup(0, 2.5), 0);
        assert_eq!(scaled_shard_pickup(1, 0.2), 1);
        assert_eq!(scaled_shard_pickup(3, 1.6), 5);
    }
}
