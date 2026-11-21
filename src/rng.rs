//! Fast deterministic PRNG (`XorShift64*`) used across gameplay simulation.
//!
//! Using an explicit RNG state inside `Game` keeps simulation deterministic for
//! unit and regression tests while remaining allocation-free and fast at runtime.

#[derive(Clone, Copy, Debug)]
pub struct GameRng {
    state: u64,
}

impl GameRng {
    /// Creates a deterministic RNG with the provided non-zero seed.
    pub fn seeded(seed: u64) -> Self {
        let s = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state: s }
    }

    /// Creates a runtime RNG seeded from system time when available.
    pub fn from_entropy() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xA5A5_5A5A_1234_5678);
            Self::seeded(nanos ^ 0xD1B5_4A32_D192_ED03)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let t = (macroquad::time::get_time() * 1_000_000.0) as u64;
            Self::seeded(t ^ 0xD1B5_4A32_D192_ED03)
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform float in `[0.0, 1.0)`.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        // Use top 24 bits for exact IEEE-754 f32 mantissa precision.
        let bits = (self.next_u64() >> 40) as u32;
        (bits as f32) * (1.0_f32 / 16_777_216.0_f32)
    }

    /// Uniform float in `[low, high)`. Returns `low` if `high <= low` or non-finite.
    #[inline]
    pub fn range_f32(&mut self, low: f32, high: f32) -> f32 {
        if !low.is_finite() || !high.is_finite() || high <= low {
            return low;
        }
        low + self.next_f32() * (high - low)
    }

    /// Uniform integer in `[low, high)`. Returns `low` if `high <= low`.
    #[inline]
    pub fn range_usize(&mut self, low: usize, high: usize) -> usize {
        if high <= low {
            return low;
        }
        let span = (high - low) as u64;
        low + (self.next_u64() % span) as usize
    }

    /// Uniform integer in `[low, high)`. Returns `low` if `high <= low`.
    #[inline]
    pub fn range_u32(&mut self, low: u32, high: u32) -> u32 {
        if high <= low {
            return low;
        }
        let span = (high - low) as u64;
        low + (self.next_u64() % span) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence_matches_for_same_seed() {
        let mut a = GameRng::seeded(42);
        let mut b = GameRng::seeded(42);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn range_bounds_are_respected() {
        let mut rng = GameRng::seeded(1337);
        for _ in 0..500 {
            let f = rng.range_f32(-10.0, 25.0);
            assert!((-10.0..25.0).contains(&f));
            let u = rng.range_usize(3, 12);
            assert!((3..12).contains(&u));
        }
        assert_eq!(rng.range_f32(5.0, 5.0), 5.0);
        assert_eq!(rng.range_usize(7, 4), 7);
    }
}
