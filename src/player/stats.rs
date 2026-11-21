//! Player hull specifications (`SHIPS`) and per-run player combat/physics state.

use crate::persistence::MetaSave;
use crate::progression::DifficultySpec;
use macroquad::color::Color;
use macroquad::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct ShipSpec {
    pub id: u32,
    pub name: &'static str,
    pub short_name: &'static str,
    pub perk: &'static str,
    pub cost: u32,
    pub rgb: [u8; 3],
}

impl ShipSpec {
    #[inline]
    pub fn color(self) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], 255)
    }

    #[inline]
    pub fn color_a(self, a: u8) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], a)
    }
}

pub const SHIPS: [ShipSpec; 4] = [
    ShipSpec {
        id: 0,
        name: "INTERCEPTOR",
        short_name: "INTERCEPTOR",
        perk: "Zbalansowane statystyki bojowe",
        cost: 0,
        rgb: [5, 217, 232],
    },
    ShipSpec {
        id: 1,
        name: "VIPER MK-II",
        short_name: "VIPER",
        perk: "+22% Spd, +28% ROF, +1 Salwa, -15% HP",
        cost: 1500,
        rgb: [5, 255, 161],
    },
    ShipSpec {
        id: 2,
        name: "TITAN DREADNOUGHT",
        short_name: "TITAN",
        perk: "+70% HP, +60 Tarcza, Strzelba 4x Plazma",
        cost: 4500,
        rgb: [249, 200, 14],
    },
    ShipSpec {
        id: 3,
        name: "PHANTOM WRAITH",
        short_name: "PHANTOM",
        perk: "-32% Dash CD, +15% CRIT, Miny po Dashu",
        cost: 9000,
        rgb: [184, 41, 234],
    },
];

pub fn ship_by_id(id: u32) -> ShipSpec {
    SHIPS[(id as usize).min(SHIPS.len() - 1)]
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub aim_angle: f32,
    pub thrust_accum: f32,

    pub dash_timer: f32,
    pub dash_duration: f32,
    pub dash_cd: f32,
    pub dash_max_cd: f32,
    pub dash_dir: Vec2,

    pub hp: f32,
    pub max_hp: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub shield_delay: f32,
    pub hp_regen: f32,
    pub crit_chance: f32,
    pub greed_mult: f32,

    pub move_speed: f32,
    pub fire_cd: f32,
    pub fire_interval: f32,
    pub bullet_dmg: f32,
    pub bullet_count: i32,
    pub bullet_pierce: i32,
    pub bullet_spread: f32,
    pub magnet_radius: f32,

    pub orbit_count: i32,
    pub orbit_angle: f32,
    pub orbit_dmg: f32,
    pub tesla_lvl: i32,
    pub tesla_timer: f32,
    pub nova_lvl: i32,
    pub nova_timer: f32,
    pub overdrive_timer: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            aim_angle: 0.0,
            thrust_accum: 0.0,
            dash_timer: 0.0,
            dash_duration: 0.18,
            dash_cd: 0.0,
            dash_max_cd: 1.6,
            dash_dir: Vec2::X,
            hp: 100.0,
            max_hp: 100.0,
            shield: 0.0,
            max_shield: 0.0,
            shield_delay: 0.0,
            hp_regen: 0.0,
            crit_chance: 0.0,
            greed_mult: 1.0,
            move_speed: 275.0,
            fire_cd: 0.0,
            fire_interval: 0.21,
            bullet_dmg: 18.0,
            bullet_count: 1,
            bullet_pierce: 1,
            bullet_spread: 0.11,
            magnet_radius: 130.0,
            orbit_count: 0,
            orbit_angle: 0.0,
            orbit_dmg: 16.0,
            tesla_lvl: 0,
            tesla_timer: 0.0,
            nova_lvl: 0,
            nova_timer: 0.0,
            overdrive_timer: 0.0,
        }
    }
}

impl PlayerState {
    /// Computes starting player stats for a new run from permanent meta upgrades, ship, and difficulty.
    pub fn from_meta(meta: &MetaSave, diff: DifficultySpec) -> Self {
        let ship_id = meta.selected_ship;
        let prestige_dmg = 1.0 + meta.prestige_lvl as f32 * 0.10;
        let prestige_hp = 1.0 + meta.prestige_lvl as f32 * 0.08;
        let prestige_shards = 1.0 + meta.prestige_lvl as f32 * 0.10;

        let dash_duration = 0.18 + meta.dash_lvl as f32 * 0.015;
        let ship_dash_mult = if ship_id == 3 { 0.68 } else { 1.0 };
        let dash_max_cd = (1.6 * 0.88_f32.powi(meta.dash_lvl as i32) * ship_dash_mult).max(0.42);

        let ship_hp_mult = if ship_id == 2 {
            1.7
        } else if ship_id == 1 {
            0.85
        } else {
            1.0
        };
        let max_hp = ((100.0 + meta.hp_lvl as f32 * 30.0) * prestige_hp * ship_hp_mult).round();
        let max_shield = meta.shield_lvl as f32 * 35.0 + if ship_id == 2 { 60.0 } else { 0.0 };
        let hp_regen = meta.regen_lvl as f32 * 1.5;
        let crit_chance =
            (meta.crit_lvl as f32 * 0.08 + if ship_id == 3 { 0.15 } else { 0.0 }).min(0.90);
        let greed_mult = (1.0 + meta.greed_lvl as f32 * 0.20) * prestige_shards * diff.shard_mult;

        let ship_dmg_mult = if ship_id == 2 { 1.35 } else { 1.0 };
        let bullet_dmg = 18.0 * (1.0 + meta.dmg_lvl as f32 * 0.15) * prestige_dmg * ship_dmg_mult;
        let ship_rof_mult = if ship_id == 1 {
            0.78
        } else if ship_id == 2 {
            1.35
        } else {
            1.0
        };
        let fire_interval = (0.21 * 0.91_f32.powi(meta.rate_lvl as i32) * ship_rof_mult).max(0.048);
        let ship_spd_mult = if ship_id == 1 {
            1.22
        } else if ship_id == 2 {
            0.86
        } else {
            1.0
        };
        let move_speed = 275.0 * (1.0 + meta.spd_lvl as f32 * 0.08) * ship_spd_mult;

        let ship_extra_bullets = if ship_id == 2 {
            3
        } else if ship_id == 1 {
            1
        } else {
            0
        };
        let bullet_count = 1 + meta.start_salvo_lvl as i32 + ship_extra_bullets;
        let bullet_pierce = 1 + (meta.start_salvo_lvl / 2) as i32;
        let bullet_spread = if ship_id == 2 { 0.165 } else { 0.11 };
        let magnet_radius = 130.0 * (1.0 + meta.mag_lvl as f32 * 0.35);

        let orbit_count = meta.drone_lvl as i32;
        let orbit_dmg = 16.0 + meta.drone_lvl as f32 * 4.0;
        let nova_lvl = meta.supernova_lvl.div_ceil(2) as i32;
        let tesla_lvl = (meta.supernova_lvl / 2) as i32;

        Self {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            aim_angle: 0.0,
            thrust_accum: 0.0,
            dash_timer: 0.0,
            dash_duration,
            dash_cd: 0.0,
            dash_max_cd,
            dash_dir: Vec2::X,
            hp: max_hp,
            max_hp,
            shield: max_shield,
            max_shield,
            shield_delay: 0.0,
            hp_regen,
            crit_chance,
            greed_mult,
            move_speed,
            fire_cd: 0.0,
            fire_interval,
            bullet_dmg,
            bullet_count,
            bullet_pierce,
            bullet_spread,
            magnet_radius,
            orbit_count,
            orbit_angle: 0.0,
            orbit_dmg,
            tesla_lvl,
            tesla_timer: 1.0,
            nova_lvl,
            nova_timer: 1.6,
            overdrive_timer: 0.0,
        }
    }

    /// Applies an in-run level-up upgrade card (`id` in `0..=11`).
    pub fn apply_upgrade(&mut self, id: u8) {
        match id {
            0 => self.bullet_count = self.bullet_count.saturating_add(1).min(24),
            1 => self.bullet_dmg = (self.bullet_dmg * 1.25).min(1_000_000.0),
            2 => self.fire_interval = (self.fire_interval * 0.83).max(0.045),
            3 => self.bullet_pierce = self.bullet_pierce.saturating_add(1).min(24),
            4 => {
                self.orbit_count = self.orbit_count.saturating_add(1).min(16);
                self.orbit_dmg += 6.0;
            }
            5 => self.tesla_lvl = self.tesla_lvl.saturating_add(1).min(20),
            6 => self.nova_lvl = self.nova_lvl.saturating_add(1).min(20),
            7 => self.dash_max_cd = (self.dash_max_cd * 0.75).max(0.45),
            8 => {
                self.max_hp += 25.0;
                self.hp = (self.hp + self.max_hp * 0.4).min(self.max_hp);
            }
            9 => self.magnet_radius = (self.magnet_radius * 1.45).min(2400.0),
            10 => self.crit_chance = (self.crit_chance + 0.15).min(0.95),
            11 => self.hp_regen += 3.0,
            _ => {}
        }
    }

    /// Ticks passive regeneration, plasma shield recharge, and overdrive duration.
    pub fn tick_vitals(&mut self, dt: f32, shield_lvl: u32) {
        if self.hp_regen > 0.0 && self.hp < self.max_hp {
            self.hp = (self.hp + self.hp_regen * dt).min(self.max_hp);
        }
        if self.max_shield > 0.0 {
            if self.shield_delay > 0.0 {
                self.shield_delay = (self.shield_delay - dt).max(0.0);
            } else if self.shield < self.max_shield {
                let s_regen = 18.0 + shield_lvl as f32 * 5.0;
                self.shield = (self.shield + s_regen * dt).min(self.max_shield);
            }
        }
        if self.overdrive_timer > 0.0 {
            self.overdrive_timer = (self.overdrive_timer - dt).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression::DIFFICULTIES;

    #[test]
    fn titan_ship_receives_bonus_hull_shield_and_quad_salvo() {
        let meta = MetaSave {
            selected_ship: 2,
            unlocked_ships_mask: 0b1111,
            ..MetaSave::default()
        };
        let p = PlayerState::from_meta(&meta, DIFFICULTIES[0]);
        assert_eq!(p.max_hp, 170.0);
        assert_eq!(p.max_shield, 60.0);
        assert_eq!(p.bullet_count, 4);
    }

    #[test]
    fn upgrades_respect_safety_caps_and_update_stats() {
        let mut p = PlayerState::default();
        for _ in 0..30 {
            p.apply_upgrade(2); // ROF
            p.apply_upgrade(7); // Dash CD
            p.apply_upgrade(10); // Crit
        }
        assert!((p.fire_interval - 0.045).abs() < 1e-5);
        assert!((p.dash_max_cd - 0.45).abs() < 1e-5);
        assert!((p.crit_chance - 0.95).abs() < 1e-5);
    }
}
