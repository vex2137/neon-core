//! Top-level screen state, expanded user settings, and wave announcement banner state.

use crate::effects::TextBuf;
use crate::game::config::MAX_PARTICLES;
use macroquad::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenState {
    Menu,
    Playing,
    Paused,
    Settings,
    LevelUp,
    GameOver { victory: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    // Audio
    pub sound: bool,
    pub master_volume: u8, // 0..=100 in 10% steps
    pub sfx_volume: u8,    // 0..=100 in 10% steps

    // Visual & Feel
    pub shake_intensity: u8, // 0 = Off, 50 = 50%, 100 = 100%
    pub hit_stop: bool,
    pub dmg_nums: bool,
    pub high_particles: bool, // false = Low (50%), true = High (100%)

    // Controls
    pub fixed_stick: bool,
    pub auto_aim: bool,
    pub auto_fire: bool,
    pub stick_deadzone: u8, // 5..=30 in 5% steps
    pub vibration: bool,

    // Diagnostics
    pub perf_hud: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: true,
            master_volume: 100,
            sfx_volume: 100,
            shake_intensity: 100,
            hit_stop: true,
            dmg_nums: true,
            high_particles: true,
            fixed_stick: false,
            auto_aim: true,
            auto_fire: true,
            stick_deadzone: 15,
            vibration: true,
            perf_hud: false,
        }
    }
}

impl Settings {
    /// Clamps and snaps all numeric setting values to valid ranges.
    #[inline]
    pub fn validated(mut self) -> Self {
        self.master_volume = ((self.master_volume.min(100) + 5) / 10) * 10;
        self.sfx_volume = ((self.sfx_volume.min(100) + 5) / 10) * 10;
        self.shake_intensity = if self.shake_intensity == 0 {
            0
        } else if self.shake_intensity <= 75 {
            50
        } else {
            100
        };
        let dz = self.stick_deadzone.clamp(5, 30);
        self.stick_deadzone = ((dz + 2) / 5) * 5;
        self.stick_deadzone = self.stick_deadzone.clamp(5, 30);
        self
    }

    /// Effective SFX gain in `0.0..=1.0`, factoring in mute toggle, master volume, and SFX volume.
    #[inline]
    pub fn effective_sfx_gain(self) -> f32 {
        if !self.sound {
            0.0
        } else {
            (self.master_volume.min(100) as f32 * 0.01) * (self.sfx_volume.min(100) as f32 * 0.01)
        }
    }

    /// Camera shake multiplier (`0.0`, `0.5`, or `1.0`).
    #[inline]
    pub fn shake_scale(self) -> f32 {
        (self.shake_intensity.min(100) as f32) * 0.01
    }

    /// Normalized analog / virtual stick deadzone in `0.05..=0.30`.
    #[inline]
    pub fn deadzone_f32(self) -> f32 {
        (self.stick_deadzone.clamp(5, 30) as f32) * 0.01
    }

    /// Active particle cap respecting the Particle Density setting (`Low` = 50%, `High` = 100%).
    #[inline]
    pub fn particle_cap(self) -> usize {
        if self.high_particles {
            MAX_PARTICLES
        } else {
            MAX_PARTICLES / 2
        }
    }

    pub fn step_master_volume(&mut self, delta: i32) {
        let next = (self.master_volume as i32 + delta * 10).clamp(0, 100) as u8;
        self.master_volume = next;
        self.sound = self.master_volume > 0;
    }

    pub fn cycle_master_volume(&mut self) {
        self.master_volume = if self.master_volume >= 100 {
            0
        } else {
            (self.master_volume + 10).min(100)
        };
        self.sound = self.master_volume > 0;
    }

    pub fn step_sfx_volume(&mut self, delta: i32) {
        self.sfx_volume = (self.sfx_volume as i32 + delta * 10).clamp(0, 100) as u8;
    }

    pub fn cycle_sfx_volume(&mut self) {
        self.sfx_volume = if self.sfx_volume >= 100 {
            0
        } else {
            (self.sfx_volume + 10).min(100)
        };
    }

    pub fn step_shake(&mut self, delta: i32) {
        let idx = match self.shake_intensity {
            0 => 0_i32,
            1..=75 => 1,
            _ => 2,
        };
        let next = (idx + delta).clamp(0, 2);
        self.shake_intensity = match next {
            0 => 0,
            1 => 50,
            _ => 100,
        };
    }

    pub fn cycle_shake(&mut self) {
        self.shake_intensity = match self.shake_intensity {
            0 => 50,
            1..=75 => 100,
            _ => 0,
        };
    }

    pub fn step_deadzone(&mut self, delta: i32) {
        self.stick_deadzone = (self.stick_deadzone as i32 + delta * 5).clamp(5, 30) as u8;
    }

    pub fn cycle_deadzone(&mut self) {
        self.stick_deadzone = if self.stick_deadzone >= 30 {
            5
        } else {
            (self.stick_deadzone + 5).min(30)
        };
    }
}

#[derive(Clone, Copy)]
pub struct BannerState {
    pub title: TextBuf<64>,
    pub subtitle: TextBuf<80>,
    pub timer: f32,
    pub color: Color,
}

impl BannerState {
    pub fn new() -> Self {
        Self {
            title: TextBuf::new(),
            subtitle: TextBuf::new(),
            timer: 0.0,
            color: Color::from_rgba(5, 217, 232, 255),
        }
    }

    pub fn show(&mut self, title: &str, subtitle: &str, duration: f32, color: Color) {
        self.title = TextBuf::from_str_trunc(title);
        self.subtitle = TextBuf::from_str_trunc(subtitle);
        self.timer = duration.max(0.0);
        self.color = color;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }
    }
}

impl Default for BannerState {
    fn default() -> Self {
        Self::new()
    }
}
