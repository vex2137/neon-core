//! Wave progression, difficulty presets, and wave event selection.

use macroquad::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct DifficultySpec {
    pub id: u32,
    pub name: &'static str,
    pub short_tag: &'static str,
    pub shard_mult: f32,
    pub hp_mult: f32,
    pub spd_mult: f32,
    pub dmg_mult: f32,
    pub elite_wave: u32,
    pub double_boss_wave: u32,
    pub rgb: [u8; 3],
}

impl DifficultySpec {
    #[inline]
    pub fn color(self) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], 255)
    }
}

pub const DIFFICULTIES: [DifficultySpec; 3] = [
    DifficultySpec {
        id: 0,
        name: "NORMAL",
        short_tag: "NORMAL 1x",
        shard_mult: 1.0,
        hp_mult: 1.0,
        spd_mult: 1.0,
        dmg_mult: 1.0,
        elite_wave: 4,
        double_boss_wave: 9,
        rgb: [5, 217, 232],
    },
    DifficultySpec {
        id: 1,
        name: "HARDCORE",
        short_tag: "HARDCORE 1.6x",
        shard_mult: 1.6,
        hp_mult: 1.45,
        spd_mult: 1.14,
        dmg_mult: 1.35,
        elite_wave: 2,
        double_boss_wave: 6,
        rgb: [255, 123, 0],
    },
    DifficultySpec {
        id: 2,
        name: "NIGHTMARE",
        short_tag: "NIGHTMARE 2.5x",
        shard_mult: 2.5,
        hp_mult: 2.2,
        spd_mult: 1.26,
        dmg_mult: 1.75,
        elite_wave: 1,
        double_boss_wave: 3,
        rgb: [255, 58, 92],
    },
];

pub const WAVE_DURATION: f32 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveEvent {
    Normal,
    Swarm,
    Orbital,
    Sniper,
    Bomber,
    Boss,
}

impl WaveEvent {
    #[cfg(any(not(target_arch = "wasm32"), test))]
    #[inline]
    pub fn as_u8(self) -> u8 {
        match self {
            WaveEvent::Normal => 0,
            WaveEvent::Swarm => 1,
            WaveEvent::Orbital => 2,
            WaveEvent::Sniper => 3,
            WaveEvent::Bomber => 4,
            WaveEvent::Boss => 5,
        }
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    #[inline]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => WaveEvent::Swarm,
            2 => WaveEvent::Orbital,
            3 => WaveEvent::Sniper,
            4 => WaveEvent::Bomber,
            5 => WaveEvent::Boss,
            _ => WaveEvent::Normal,
        }
    }

    #[inline]
    pub fn tag(self) -> &'static str {
        match self {
            WaveEvent::Normal => "",
            WaveEvent::Swarm => " [SWARM]",
            WaveEvent::Orbital => " [ORBITAL]",
            WaveEvent::Sniper => " [SNIPER]",
            WaveEvent::Bomber => " [BOMBER]",
            WaveEvent::Boss => " [BOSS]",
        }
    }
}

/// Determines the wave event type for `wave >= 2`.
#[inline]
pub fn event_for_wave(wave: u32) -> WaveEvent {
    if wave.is_multiple_of(3) {
        WaveEvent::Boss
    } else if wave >= 4 {
        match wave % 4 {
            0 => WaveEvent::Swarm,
            1 => WaveEvent::Orbital,
            2 => WaveEvent::Sniper,
            _ => WaveEvent::Bomber,
        }
    } else {
        match wave % 3 {
            0 => WaveEvent::Sniper,
            1 => WaveEvent::Orbital,
            _ => WaveEvent::Swarm,
        }
    }
}

/// Calculates how many Overlord Bosses spawn on a boss wave for the active difficulty.
#[inline]
pub fn boss_count_for_wave(wave: u32, diff: DifficultySpec) -> u32 {
    if wave >= diff.double_boss_wave.saturating_add(9) {
        4
    } else if wave >= diff.double_boss_wave.saturating_add(6) {
        3
    } else if wave >= diff.double_boss_wave {
        2
    } else {
        1
    }
}

/// Computes enemy spawn rate (enemies/sec) for the active wave, event, and difficulty.
#[inline]
pub fn wave_spawn_rate(wave: u32, event: WaveEvent, diff: DifficultySpec) -> f32 {
    let event_rate_mult = match event {
        WaveEvent::Swarm => 1.55,
        WaveEvent::Bomber => 1.25,
        _ => 1.0,
    };
    let diff_rate_mult = 1.0 + diff.id as f32 * 0.22;
    ((2.0 + wave as f32 * 0.72) * event_rate_mult * diff_rate_mult).min(18.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_event_schedule_matches_expected_cycle() {
        assert_eq!(event_for_wave(2), WaveEvent::Swarm);
        assert_eq!(event_for_wave(3), WaveEvent::Boss);
        assert_eq!(event_for_wave(4), WaveEvent::Swarm);
        assert_eq!(event_for_wave(5), WaveEvent::Orbital);
        assert_eq!(event_for_wave(6), WaveEvent::Boss);
        assert_eq!(event_for_wave(7), WaveEvent::Bomber);
        assert_eq!(event_for_wave(10), WaveEvent::Sniper);
    }

    #[test]
    fn boss_count_scales_with_difficulty_and_caps_at_four() {
        let normal = DIFFICULTIES[0];
        let nightmare = DIFFICULTIES[2];
        assert_eq!(boss_count_for_wave(3, normal), 1);
        assert_eq!(boss_count_for_wave(9, normal), 2);
        assert_eq!(boss_count_for_wave(15, normal), 3);
        assert_eq!(boss_count_for_wave(18, normal), 4);
        assert_eq!(boss_count_for_wave(3, nightmare), 2);
        assert_eq!(boss_count_for_wave(12, nightmare), 4);
    }

    #[test]
    fn spawn_rate_is_capped_at_eighteen() {
        let rate = wave_spawn_rate(100, WaveEvent::Swarm, DIFFICULTIES[2]);
        assert!((rate - 18.0).abs() < 1e-4);
    }
}
