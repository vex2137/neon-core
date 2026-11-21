//! Run lifecycle transitions: starting runs, pausing/resuming, wave transitions, and finishing runs.

use super::config::{Viewport, ARENA_RADIUS, DEMO_MAX_WAVE};
use super::state::ScreenState;
use super::Game;
use crate::effects::TextBuf;
use crate::enemies::EnemyKind;
use crate::progression::{
    boss_count_for_wave, event_for_wave, next_level_xp, roll_three_upgrades, WaveEvent,
};
use crate::world::clamp_to_arena;
use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};
use std::fmt::Write;

impl Game {
    pub fn start_run(&mut self, vp: Viewport) {
        self.bullets.clear();
        self.e_bullets.clear();
        self.hazards.clear();
        self.mines.clear();
        self.enemies.clear();
        self.enemies_dirty = false;
        self.gems.clear();
        self.effects.clear();
        self.touch.clear();

        let diff = self.difficulty();
        let ship = self.selected_ship();
        self.player = crate::player::PlayerState::from_meta(&self.meta, diff);

        self.wave = 1;
        self.wave_timer = 0.0;
        self.wave_event = WaveEvent::Normal;

        let mut title = TextBuf::<64>::new();
        let _ = write!(&mut title, "FALA 1 - {} [{}]", ship.name, diff.name);
        let mut sub = TextBuf::<80>::new();
        if diff.id == 0 {
            sub.push_str("ZBIERAJ PRZEKAZNIKI I UNIKAJ OGNIA");
        } else {
            let _ = write!(
                &mut sub,
                "MNOZNIK SHARDOW {:.1}x - EKSTREMALNE ZAGROZENIE",
                diff.shard_mult
            );
        }
        self.banner
            .show(title.as_str(), sub.as_str(), 3.2, diff.color());

        self.hazard_spawn_timer = if diff.id == 2 { 3.2 } else { 5.0 };
        self.beacon_spawn_timer = 14.0;
        self.beacon.reset();
        self.combo = 0;
        self.combo_timer = 0.0;
        self.spawn_accum = 0.0;
        self.level = 1;
        self.xp = 0.0;
        self.next_xp = 25.0;
        self.run_shards = 0;
        self.kills = 0;
        self.is_new_record = false;
        self.xp_popup_accum = 0.0;
        self.shard_popup_accum = 0;
        self.pickup_popup_timer = 0.0;

        for _ in 0..(8 + diff.id * 2) {
            self.spawn_enemy(EnemyKind::Swarmer, vp);
        }
        self.state = ScreenState::Playing;
    }

    pub fn pause_run(&mut self) {
        if self.state == ScreenState::Playing {
            self.touch.clear();
            self.state = ScreenState::Paused;
        }
    }

    pub fn resume_run(&mut self) {
        self.touch.clear();
        self.state = ScreenState::Playing;
    }

    pub fn roll_levelup(&mut self) {
        self.touch.clear();
        self.level_choices = roll_three_upgrades(&mut self.rng);
        self.state = ScreenState::LevelUp;
    }

    pub fn apply_upgrade(&mut self, id: u8) {
        self.player.apply_upgrade(id);
        self.touch.clear();
        self.state = ScreenState::Playing;
    }

    pub fn finish_run(&mut self, victory: bool) {
        self.touch.clear();
        self.meta.shards = self.meta.shards.saturating_add(self.run_shards);
        self.is_new_record = self.wave > self.meta.best_wave || self.kills > self.meta.best_kills;
        if self.wave > self.meta.best_wave {
            self.meta.best_wave = self.wave;
        }
        if self.kills > self.meta.best_kills {
            self.meta.best_kills = self.kills;
        }
        if self.persist_saves {
            self.meta.save();
        }
        self.state = ScreenState::GameOver { victory };
    }

    pub fn advance_wave(&mut self, vp: Viewport) {
        self.wave = self.wave.saturating_add(1);
        if cfg!(feature = "demo") && self.wave > DEMO_MAX_WAVE {
            self.finish_run(true);
            return;
        }

        let w = self.wave;
        let diff = self.difficulty();
        let ev = event_for_wave(w);
        self.wave_event = ev;

        let mut title = TextBuf::<64>::new();
        match ev {
            WaveEvent::Boss => {
                let boss_cnt = boss_count_for_wave(w, diff);
                let _ = write!(
                    &mut title,
                    "FALA {} - ALARM: OVERLORD BOSS x{}!",
                    w, boss_cnt
                );
                let sub = if boss_cnt >= 2 {
                    "WIELU BOSSOW + ESKORTA SNAJPEROW"
                } else {
                    "UNIKAJ FAL POCISKOW - DASH DAJE NIESMIERTELNOSC"
                };
                self.banner.show(
                    title.as_str(),
                    sub,
                    3.6,
                    Color::from_rgba(184, 41, 234, 255),
                );
                for _ in 0..boss_cnt {
                    self.spawn_enemy(EnemyKind::Boss, vp);
                }
                for _ in 0..(2 + w / 4 + diff.id) {
                    self.spawn_enemy(EnemyKind::Sniper, vp);
                }
            }
            WaveEvent::Swarm => {
                let _ = write!(&mut title, "FALA {} - ROJ ZASADZKI!", w);
                self.banner.show(
                    title.as_str(),
                    "OTACZAJA CIE SZYBKIE JEDNOSTKI",
                    3.6,
                    Color::from_rgba(255, 123, 0, 255),
                );
                let ring_cnt = (9 + w).min(20);
                for k in 0..ring_cnt {
                    let a = (k as f32 * std::f32::consts::TAU) / ring_cnt as f32;
                    let pos = clamp_to_arena(self.player.pos + Vec2::from_angle(a) * 430.0, 50.0);
                    let kind = if k.is_multiple_of(3) {
                        EnemyKind::Dasher
                    } else {
                        EnemyKind::Swarmer
                    };
                    self.spawn_enemy_at(kind, pos, true);
                }
            }
            WaveEvent::Orbital => {
                let _ = write!(&mut title, "FALA {} - OSTRZAL ORBITALNY!", w);
                self.banner.show(
                    title.as_str(),
                    "UCIEKAJ Z CZERWONYCH STREF UDERZENIA",
                    3.6,
                    Color::from_rgba(255, 58, 92, 255),
                );
                self.hazard_spawn_timer = 0.55;
            }
            WaveEvent::Sniper => {
                let _ = write!(&mut title, "FALA {} - ESKADRA SNAJPEROW!", w);
                self.banner.show(
                    title.as_str(),
                    "UWAGA NA LASERY NAMIERZAJACE I WIRY",
                    3.6,
                    Color::from_rgba(125, 110, 255, 255),
                );
                for k in 0..(4 + w / 2) {
                    let kind = if k.is_multiple_of(3) && w >= 5 {
                        EnemyKind::Vortex
                    } else {
                        EnemyKind::Sniper
                    };
                    self.spawn_enemy(kind, vp);
                }
            }
            WaveEvent::Bomber | WaveEvent::Normal => {
                let _ = write!(&mut title, "FALA {} - FALA BOMBOWCOW!", w);
                self.banner.show(
                    title.as_str(),
                    "NISZCZ Z DYSTANSU - EKSPLODUJA ODLAMKAMI",
                    3.6,
                    Color::from_rgba(170, 255, 45, 255),
                );
                for _ in 0..(5 + w / 2) {
                    self.spawn_enemy(EnemyKind::Bomber, vp);
                }
            }
        }
        self.sounds
            .play(&self.sounds.lvlup, self.settings.sound, 0.55);
    }

    pub fn complete_beacon(&mut self) {
        let bpos = self.beacon.pos;
        self.beacon.reset();
        self.beacon_spawn_timer = 22.0;

        let od_dur = 5.5 + self.meta.overdrive_lvl as f32 * 1.2;
        self.player.overdrive_timer = self.player.overdrive_timer.max(od_dur);

        let bonus_shards = ((6 + self.wave * 2) as f32 * self.player.greed_mult).round() as u32;
        let bonus_xp = (22.0 + self.wave as f32 * 6.0) * self.player.greed_mult;
        self.run_shards = self.run_shards.saturating_add(bonus_shards);
        self.xp += bonus_xp;

        let clear_r2 = 480.0 * 480.0;
        self.e_bullets
            .retain(|eb| eb.pos.distance_squared(bpos) > clear_r2);

        let gold = Color::from_rgba(249, 200, 14, 255);
        self.effects.spawn_ring(bpos, 360.0, gold, 0.5);
        let mut msg = TextBuf::<32>::new();
        let _ = write!(&mut msg, "OVERDRIVE! +{} SHARDS", bonus_shards);
        self.effects.spawn_text(bpos + vec2(0.0, -28.0), msg, gold);
        self.sounds
            .play(&self.sounds.lvlup, self.settings.sound, 0.95);
        self.effects.add_shake(7.0, self.settings.shake);
        self.effects.add_hit_stop(0.045);

        if self.xp >= self.next_xp {
            self.xp -= self.next_xp;
            self.level = self.level.saturating_add(1);
            self.next_xp = next_level_xp(self.next_xp);
            self.roll_levelup();
        }
    }

    #[allow(dead_code)]
    pub(crate) fn arena_radius() -> f32 {
        ARENA_RADIUS
    }
}
