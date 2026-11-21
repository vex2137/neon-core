//! Enemy archetype definitions and runtime state.

use macroquad::color::Color;
use macroquad::math::Vec2;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Swarmer = 0,
    Dasher = 1,
    Tank = 2,
    Boss = 3,
    Sniper = 4,
    Bomber = 5,
    Vortex = 6,
}

impl EnemyKind {
    #[cfg(any(not(target_arch = "wasm32"), test))]
    #[inline]
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => EnemyKind::Swarmer,
            1 => EnemyKind::Dasher,
            2 => EnemyKind::Tank,
            3 => EnemyKind::Boss,
            4 => EnemyKind::Sniper,
            5 => EnemyKind::Bomber,
            _ => EnemyKind::Vortex,
        }
    }

    #[inline]
    pub fn base_color(self, enraged_boss: bool) -> Color {
        match self {
            EnemyKind::Dasher => Color::from_rgba(255, 123, 0, 255),
            EnemyKind::Tank => Color::from_rgba(249, 200, 14, 255),
            EnemyKind::Boss => {
                if enraged_boss {
                    Color::from_rgba(255, 58, 92, 255)
                } else {
                    Color::from_rgba(184, 41, 234, 255)
                }
            }
            EnemyKind::Sniper => Color::from_rgba(255, 58, 92, 255),
            EnemyKind::Bomber => Color::from_rgba(170, 255, 45, 255),
            EnemyKind::Vortex => Color::from_rgba(125, 110, 255, 255),
            EnemyKind::Swarmer => Color::from_rgba(255, 42, 109, 255),
        }
    }

    #[inline]
    pub fn polygon_sides(self) -> u8 {
        match self {
            EnemyKind::Swarmer => 3,
            EnemyKind::Dasher | EnemyKind::Bomber => 4,
            EnemyKind::Sniper => 5,
            EnemyKind::Tank => 6,
            EnemyKind::Vortex => 7,
            EnemyKind::Boss => 8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub alive: bool,
    pub kind: EnemyKind,
    pub elite: bool,
    pub pos: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub radius: f32,
    pub speed: f32,
    pub flash: f32,
    pub timer: f32,
    pub shoot_cd: f32,
    pub angle: f32,
    pub spin: f32,
}
