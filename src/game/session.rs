//! Run lifecycle transitions: starting runs, mid-run save/resume, pausing, wave transitions, and finishing runs.

use super::config::{Viewport, ARENA_RADIUS, DEMO_MAX_WAVE};
use super::state::ScreenState;
use super::Game;
use crate::effects::TextBuf;
use crate::enemies::EnemyKind;
use crate::persistence::{RunSnapshot, RunSummary};
use crate::progression::{
    boss_count_for_wave, event_for_wave, next_level_xp, roll_three_upgrades, WaveEvent,
};
use crate::rng::GameRng;
use crate::world::clamp_to_arena;
use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};
use std::fmt::Write;

impl Game {
    pub fn start_run(&mut self, vp: Viewport) {
        // Starting a fresh run clears any previously saved mid-run state.
        self.delete_saved_run();

        self.bullets.clear();
        self.e_bullets.clear();
        self.hazards.clear();
        self.mines.clear();
        self.enemies.clear();
        self.enemies_dirty = false;
        self.gems.clear();
        self.effects.clear();
        self.effects.set_particle_cap(self.settings.particle_cap());
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

    /// Captures a complete, deterministic mid-run snapshot for serialization.
    pub fn capture_run_snapshot(&self) -> RunSnapshot {
        RunSnapshot {
            ship_id: self.meta.selected_ship,
            difficulty: self.meta.difficulty,
            wave: self.wave,
            wave_timer: self.wave_timer,
            wave_event: self.wave_event,
            rng_state: self.rng.state(),
            xp: self.xp,
            next_xp: self.next_xp,
            level: self.level,
            combo: self.combo,
            combo_timer: self.combo_timer,
            spawn_accum: self.spawn_accum,
            run_shards: self.run_shards,
            kills: self.kills,
            hazard_spawn_timer: self.hazard_spawn_timer,
            beacon_spawn_timer: self.beacon_spawn_timer,
            beacon: self.beacon,
            player: self.player,
            enemies: self.enemies.iter().copied().filter(|e| e.alive).collect(),
            bullets: self.bullets.clone(),
            e_bullets: self.e_bullets.clone(),
            hazards: self.hazards.clone(),
            mines: self.mines.clone(),
            gems: self.gems.clone(),
        }
    }

    /// Restores a previously saved mid-run snapshot and enters active gameplay.
    pub fn restore_run_snapshot(&mut self, snap: RunSnapshot) {
        let snap = snap.validated();
        self.meta.selected_ship = snap.ship_id;
        self.meta.difficulty = snap.difficulty;
        self.wave = snap.wave;
        self.wave_timer = snap.wave_timer;
        self.wave_event = snap.wave_event;
        self.rng = GameRng::seeded(snap.rng_state);
        self.xp = snap.xp;
        self.next_xp = snap.next_xp;
        self.level = snap.level;
        self.combo = snap.combo;
        self.combo_timer = snap.combo_timer;
        self.spawn_accum = snap.spawn_accum;
        self.run_shards = snap.run_shards;
        self.kills = snap.kills;
        self.hazard_spawn_timer = snap.hazard_spawn_timer;
        self.beacon_spawn_timer = snap.beacon_spawn_timer;
        self.beacon = snap.beacon;
        self.player = snap.player;
        self.enemies = snap.enemies;
        self.enemies_dirty = false;
        self.bullets = snap.bullets;
        self.e_bullets = snap.e_bullets;
        self.hazards = snap.hazards;
        self.mines = snap.mines;
        self.gems = snap.gems;
        self.effects.clear();
        self.effects.set_particle_cap(self.settings.particle_cap());
        self.touch.clear();
        self.is_new_record = false;
        self.xp_popup_accum = 0.0;
        self.shard_popup_accum = 0;
        self.pickup_popup_timer = 0.0;
        self.rebuild_spatial_grid();

        let ship = self.selected_ship();
        let diff = self.difficulty();
        let mut title = TextBuf::<64>::new();
        let _ = write!(
            &mut title,
            "WZNOWIONO - FALA {} ({})",
            self.wave, ship.short_name
        );
        self.banner
            .show(title.as_str(), "KONTYNUACJA MISJI", 2.4, diff.color());
        self.state = ScreenState::Playing;
    }

    /// Saves the active run to disk (when persistence is enabled) and updates cached menu summary.
    pub fn save_active_run(&mut self) {
        let snap = self.capture_run_snapshot();
        let summary = snap.summary();
        if self.persist_saves {
            snap.save_default();
        }
        self.saved_run_summary = Some(summary);
    }

    /// Saves the active run and returns to the Hangar/Main Menu.
    pub fn save_and_quit_to_menu(&mut self) {
        self.touch.clear();
        self.save_active_run();
        self.ui_focus.menu_idx = 0;
        self.state = ScreenState::Menu;
    }

    /// Loads `neon_core_run.dat` from disk and resumes it. Returns `true` if a valid save existed.
    pub fn resume_saved_run(&mut self) -> bool {
        if let Some(snap) = RunSnapshot::load_default() {
            self.restore_run_snapshot(snap);
            true
        } else {
            self.saved_run_summary = None;
            false
        }
    }

    /// Deletes any mid-run save file (called on run end or starting a new run).
    pub fn delete_saved_run(&mut self) {
        if self.persist_saves {
            RunSnapshot::delete_default();
        }
        self.saved_run_summary = None;
    }

    /// Refreshes the cached `RunSummary` from disk without blocking hot paths.
    #[allow(dead_code)]
    pub fn refresh_saved_run_summary(&mut self) -> Option<RunSummary> {
        if self.persist_saves {
            self.saved_run_summary = RunSnapshot::load_summary_default();
        }
        self.saved_run_summary
    }

    /// Syncs expanded `Settings` into `MetaSave`, updates live effect caps, and persists to disk.
    pub fn commit_settings(&mut self) {
        self.settings = self.settings.validated();
        self.effects.set_particle_cap(self.settings.particle_cap());
        self.meta.settings = self.settings;
        if self.persist_saves {
            self.meta.save();
        }
    }

    pub fn pause_run(&mut self) {
        if self.state == ScreenState::Playing {
            self.touch.clear();
            self.save_active_run();
            self.ui_focus.pause_idx = 0;
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
        self.ui_focus.levelup_idx = 0;
        self.state = ScreenState::LevelUp;
    }

    pub fn apply_upgrade(&mut self, id: u8) {
        self.player.apply_upgrade(id);
        self.touch.clear();
        self.state = ScreenState::Playing;
    }

    pub fn finish_run(&mut self, victory: bool) {
        self.touch.clear();
        // Immediately delete mid-run save on death/completion to prevent save-scumming.
        self.delete_saved_run();

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
            .play(&self.sounds.lvlup, self.settings.effective_sfx_gain(), 0.55);
        // Auto-save checkpoint on every wave transition.
        self.save_active_run();
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
            .play(&self.sounds.lvlup, self.settings.effective_sfx_gain(), 0.95);
        self.effects.add_shake(7.0, self.settings.shake_scale());
        self.effects.add_hit_stop(0.045, self.settings.hit_stop);
        self.gamepad
            .trigger_rumble(0.55, 110, self.settings.vibration);

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
