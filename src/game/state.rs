//! Top-level screen state, user settings, and wave announcement banner state.

use crate::effects::TextBuf;
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
    pub sound: bool,
    pub shake: bool,
    pub dmg_nums: bool,
    pub fixed_stick: bool,
    pub auto_aim: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: true,
            shake: true,
            dmg_nums: true,
            fixed_stick: false,
            auto_aim: true,
        }
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
