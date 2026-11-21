//! Top-level `Game` state container and deterministic per-frame simulation orchestrator.

pub mod config;
pub mod session;
pub mod state;

pub use config::{
    Viewport, ARENA_RADIUS, MAX_BULLETS, MAX_EBULLETS, MAX_ENEMIES, MAX_GEMS, MAX_HAZARDS,
    MAX_MINES,
};
pub use state::{BannerState, ScreenState, Settings};

use crate::audio::SoundBank;
use crate::combat::{
    apply_player_damage, roll_hit_damage, roll_kill_reward, update_enemy_bullets, update_gems,
    update_hazards, Bullet, EnemyBullet, Mine,
};
use crate::effects::{EffectsPool, TextBuf};
use crate::enemies::{
    compute_spawn_position, create_enemy, roll_wave_enemy_kind, update_enemies_ai, Enemy, EnemyKind,
};
use crate::input::{FrameInput, TouchState};
use crate::performance::PerfMetrics;
use crate::persistence::MetaSave;
use crate::player::{
    ship_by_id, try_trigger_dash, update_aim, update_player_movement, update_primary_weapon,
    PlayerState, ShipSpec,
};
use crate::progression::{
    next_level_xp, wave_spawn_rate, DifficultySpec, Gem, UpgradeCard, WaveEvent, ALL_UPGRADES,
    DIFFICULTIES, WAVE_DURATION,
};
use crate::rng::GameRng;
use crate::world::{
    clamp_to_arena, generate_starfield, tick_hazard_spawner, BeaconObjective, BeaconTickEvent,
    HazardZone, SpatialGrid, Star,
};
use macroquad::color::{Color, WHITE};
use macroquad::math::{vec2, Vec2};
use std::fmt::Write;

pub struct Game {
    pub state: ScreenState,
    pub prev_state: ScreenState,
    pub shop_tab: u8,
    pub settings: Settings,
    pub meta: MetaSave,
    pub persist_saves: bool,
    pub sounds: SoundBank,
    pub rng: GameRng,
    pub metrics: PerfMetrics,

    pub bullets: Vec<Bullet>,
    pub e_bullets: Vec<EnemyBullet>,
    pub hazards: Vec<HazardZone>,
    pub mines: Vec<Mine>,
    pub enemies: Vec<Enemy>,
    pub enemies_dirty: bool,
    pub grid: SpatialGrid,
    pub gems: Vec<Gem>,
    pub stars: Vec<Star>,
    pub effects: EffectsPool,

    pub player: PlayerState,
    pub xp: f32,
    pub next_xp: f32,
    pub level: u32,
    pub wave: u32,
    pub wave_timer: f32,
    pub wave_event: WaveEvent,
    pub banner: BannerState,
    pub hazard_spawn_timer: f32,
    pub beacon_spawn_timer: f32,
    pub beacon: BeaconObjective,
    pub combo: u32,
    pub combo_timer: f32,
    pub spawn_accum: f32,
    pub run_shards: u32,
    pub kills: u32,
    pub is_new_record: bool,
    pub xp_popup_accum: f32,
    pub shard_popup_accum: u32,
    pub pickup_popup_timer: f32,

    pub touch: TouchState,
    pub level_choices: [UpgradeCard; 3],
}

impl Game {
    pub fn new(sounds: SoundBank) -> Self {
        let mut rng = GameRng::from_entropy();
        let meta = MetaSave::load();
        Self::build(sounds, meta, true, &mut rng)
    }

    /// Deterministic constructor for headless unit/regression tests and benchmarks (never touches disk).
    #[cfg(test)]
    pub fn new_headless(seed: u64, meta: MetaSave) -> Self {
        let mut rng = GameRng::seeded(seed);
        Self::build(SoundBank::empty(), meta.validated(), false, &mut rng)
    }

    fn build(sounds: SoundBank, meta: MetaSave, persist_saves: bool, rng: &mut GameRng) -> Self {
        let stars = generate_starfield(rng, 160);
        Self {
            state: ScreenState::Menu,
            prev_state: ScreenState::Menu,
            shop_tab: 0,
            settings: Settings::default(),
            meta,
            persist_saves,
            sounds,
            rng: *rng,
            metrics: PerfMetrics::new(),

            bullets: Vec::with_capacity(MAX_BULLETS),
            e_bullets: Vec::with_capacity(MAX_EBULLETS),
            hazards: Vec::with_capacity(MAX_HAZARDS),
            mines: Vec::with_capacity(MAX_MINES),
            enemies: Vec::with_capacity(MAX_ENEMIES),
            enemies_dirty: false,
            grid: SpatialGrid::with_capacity(MAX_ENEMIES),
            gems: Vec::with_capacity(MAX_GEMS),
            stars,
            effects: EffectsPool::new(),

            player: PlayerState::default(),
            xp: 0.0,
            next_xp: 25.0,
            level: 1,
            wave: 1,
            wave_timer: 0.0,
            wave_event: WaveEvent::Normal,
            banner: BannerState::new(),
            hazard_spawn_timer: 5.0,
            beacon_spawn_timer: 14.0,
            beacon: BeaconObjective::default(),
            combo: 0,
            combo_timer: 0.0,
            spawn_accum: 0.0,
            run_shards: 0,
            kills: 0,
            is_new_record: false,
            xp_popup_accum: 0.0,
            shard_popup_accum: 0,
            pickup_popup_timer: 0.0,

            touch: TouchState::default(),
            level_choices: [ALL_UPGRADES[0], ALL_UPGRADES[1], ALL_UPGRADES[2]],
        }
    }

    #[inline]
    pub fn difficulty(&self) -> DifficultySpec {
        DIFFICULTIES[(self.meta.difficulty as usize).min(DIFFICULTIES.len() - 1)]
    }

    #[inline]
    pub fn selected_ship(&self) -> ShipSpec {
        ship_by_id(self.meta.selected_ship)
    }

    pub fn rebuild_spatial_grid(&mut self) {
        self.grid.rebuild(
            self.enemies
                .iter()
                .enumerate()
                .map(|(i, e)| (i, e.alive, e.pos)),
        );
    }

    pub fn compact_dead_enemies(&mut self) {
        if self.enemies_dirty {
            self.enemies.retain(|e| e.alive);
            self.enemies_dirty = false;
        }
    }

    pub fn find_nearest_enemy_spatial(
        &self,
        origin: Vec2,
        max_range: f32,
        exclude: &[usize],
    ) -> Option<usize> {
        self.grid.find_nearest(origin, max_range, |idx| {
            let e = &self.enemies[idx];
            if e.alive && !exclude.contains(&idx) {
                Some(e.pos)
            } else {
                None
            }
        })
    }

    /// Pure simulation tick decoupled from Macroquad input polling so tests can step frames directly.
    pub fn step_playing(&mut self, dt: f32, input: FrameInput, vp: Viewport) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }

        if input.pause_pressed {
            self.pause_run();
            return;
        }
        if input.dash_pressed {
            if let Some((pos, dmg, rad)) = try_trigger_dash(
                &mut self.player,
                input.move_dir,
                self.meta.selected_ship,
                &self.sounds,
                &mut self.effects,
                self.settings.sound,
                self.settings.shake,
            ) {
                self.spawn_mine(pos, dmg, rad);
            }
        }

        if self.effects.consume_hit_stop(dt) {
            return;
        }

        let diff = self.difficulty();
        self.player.tick_vitals(dt, self.meta.shield_lvl);
        self.banner.tick(dt);
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        // Wave progression (22s waves)
        self.wave_timer += dt;
        if self.wave_timer >= WAVE_DURATION {
            self.wave_timer = 0.0;
            self.advance_wave(vp);
            if self.state != ScreenState::Playing {
                return;
            }
        }

        // Dynamic wave spawner
        let spawn_rate = wave_spawn_rate(self.wave, self.wave_event, diff);
        self.spawn_accum += dt * spawn_rate;
        while self.spawn_accum >= 1.0 {
            self.spawn_accum -= 1.0;
            let kind = roll_wave_enemy_kind(self.wave, self.wave_event, &mut self.rng);
            self.spawn_enemy(kind, vp);
        }

        // Orbital Hazard Strikes
        tick_hazard_spawner(
            &mut self.hazards,
            &mut self.hazard_spawn_timer,
            dt,
            self.wave,
            self.wave_event,
            diff,
            self.player.pos,
            self.player.vel,
            &mut self.rng,
        );

        // Overload Beacon objective
        match self.beacon.tick(
            &mut self.beacon_spawn_timer,
            dt,
            self.player.pos,
            &mut self.rng,
        ) {
            BeaconTickEvent::Spawned => {
                if self.banner.timer <= 0.4 {
                    self.banner.show(
                        "PRZEKAZNIK MOCY AKTYWNY",
                        "PRZEJMIJ ZLOTY PUNKT NA RADARZE - OVERDRIVE + SHARDS",
                        3.0,
                        Color::from_rgba(249, 200, 14, 255),
                    );
                }
            }
            BeaconTickEvent::Captured => {
                self.complete_beacon();
                if self.state != ScreenState::Playing {
                    return;
                }
            }
            BeaconTickEvent::None | BeaconTickEvent::Expired => {}
        }

        // Player movement & spatial grid rebuild (once per frame)
        update_player_movement(
            &mut self.player,
            input.move_dir,
            dt,
            &mut self.effects,
            &mut self.rng,
        );
        self.rebuild_spatial_grid();

        // Aiming & Primary weapon
        let nearest_pos = self
            .find_nearest_enemy_spatial(self.player.pos, 620.0, &[])
            .map(|idx| self.enemies[idx].pos);
        update_aim(
            &mut self.player,
            input.manual_aim_dir,
            nearest_pos,
            self.settings.auto_aim,
        );
        update_primary_weapon(
            &mut self.player,
            &mut self.bullets,
            &mut self.sounds,
            self.settings.sound,
            self.settings.auto_aim,
            input.manual_aim_dir.is_some(),
            nearest_pos.is_some(),
            self.meta.overdrive_lvl,
            dt,
        );

        // Orbital saws
        if self.player.orbit_count > 0 {
            self.player.orbit_angle += dt * 3.6;
            let count = self.player.orbit_count;
            let orbit_dmg = self.player.orbit_dmg * dt * 8.0;
            let mut hit_buf = [0usize; 32];
            for i in 0..count {
                let a = self.player.orbit_angle + (i as f32 * std::f32::consts::TAU) / count as f32;
                let bpos = self.player.pos + Vec2::from_angle(a) * 74.0;
                let mut hit_len = 0_usize;
                self.grid.for_each_in_aabb(bpos, 52.0, |idx| {
                    let e = &self.enemies[idx];
                    if e.alive {
                        let rr = e.radius + 12.0;
                        if e.pos.distance_squared(bpos) < rr * rr && hit_len < hit_buf.len() {
                            hit_buf[hit_len] = idx;
                            hit_len += 1;
                        }
                    }
                    true
                });
                for &idx in &hit_buf[..hit_len] {
                    self.damage_enemy(idx, orbit_dmg, bpos, false);
                }
            }
        }

        // Chain Tesla
        if self.player.tesla_lvl > 0 {
            self.player.tesla_timer -= dt;
            if self.player.tesla_timer <= 0.0 {
                self.player.tesla_timer = (1.4 - self.player.tesla_lvl as f32 * 0.18).max(0.45);
                self.trigger_tesla();
            }
        }

        // EMP Nova
        if self.player.nova_lvl > 0 {
            self.player.nova_timer -= dt;
            if self.player.nova_timer <= 0.0 {
                self.player.nova_timer = (4.5 - self.player.nova_lvl as f32 * 0.45).max(1.8);
                self.trigger_nova();
            }
        }

        // Player bullets + spatial grid collisions
        let arena_r2 = ARENA_RADIUS * ARENA_RADIUS;
        let mut bi = self.bullets.len();
        let mut bullet_hits = [0usize; 16];
        while bi > 0 {
            bi -= 1;
            let vel = self.bullets[bi].vel;
            self.bullets[bi].pos += vel * dt;
            self.bullets[bi].life -= dt;
            let bpos = self.bullets[bi].pos;
            if self.bullets[bi].life <= 0.0 || bpos.length_squared() > arena_r2 {
                self.bullets.swap_remove(bi);
                continue;
            }
            let bdmg = self.bullets[bi].dmg;
            let pierce = self.bullets[bi].pierce.max(1) as usize;
            let max_hits = pierce.min(bullet_hits.len());
            let mut hit_len = 0_usize;

            self.grid.for_each_in_aabb(bpos, 45.0, |idx| {
                let e = &self.enemies[idx];
                if e.alive {
                    let rr = e.radius + 7.0;
                    if e.pos.distance_squared(bpos) < rr * rr {
                        bullet_hits[hit_len] = idx;
                        hit_len += 1;
                        if hit_len >= max_hits {
                            return false;
                        }
                    }
                }
                true
            });

            if hit_len > 0 {
                for &idx in &bullet_hits[..hit_len] {
                    self.damage_enemy(idx, bdmg, bpos, true);
                }
                self.bullets[bi].pierce -= hit_len as i32;
                if self.bullets[bi].pierce <= 0 {
                    self.bullets.swap_remove(bi);
                }
            }
        }

        // Phantom Wraith proximity plasma mines
        let mut mi = self.mines.len();
        while mi > 0 {
            mi -= 1;
            self.mines[mi].life -= dt;
            let mpos = self.mines[mi].pos;
            let near = self.find_nearest_enemy_spatial(mpos, 62.0, &[]).is_some();
            if near || self.mines[mi].life <= 0.0 {
                self.detonate_mine(mi);
            }
        }

        // Single conditional compaction pass before AI update, then AI update
        self.compact_dead_enemies();
        let ai_outcome = update_enemies_ai(
            &mut self.enemies,
            &mut self.e_bullets,
            &mut self.effects,
            &self.sounds,
            self.settings.sound,
            self.player.pos,
            self.player.dash_timer > 0.0,
            self.wave,
            diff,
            dt,
        );
        if ai_outcome.any_died {
            self.enemies_dirty = true;
        }
        if ai_outcome.player_pull.length_squared() > 0.0 {
            self.player.pos = clamp_to_arena(self.player.pos + ai_outcome.player_pull, 18.0);
        }
        if ai_outcome.player_damage > 0.0
            && self.damage_player(ai_outcome.player_damage, ai_outcome.player_shake)
        {
            return;
        }
        self.compact_dead_enemies();

        // Enemy bullets vs player
        let eb_hits = update_enemy_bullets(
            &mut self.e_bullets,
            self.player.pos,
            self.player.dash_timer > 0.0,
            dt,
            &mut self.effects,
            &mut self.rng,
        );
        if eb_hits.damage > 0.0 && self.damage_player(eb_hits.damage, eb_hits.shake) {
            return;
        }

        // Orbital Hazard detonations vs player
        let hz_hits = update_hazards(
            &mut self.hazards,
            self.player.pos,
            self.player.dash_timer > 0.0,
            dt,
            &mut self.effects,
            &self.sounds,
            self.settings.sound,
            &mut self.rng,
        );
        if hz_hits.damage > 0.0 && self.damage_player(hz_hits.damage, hz_hits.shake) {
            return;
        }

        // Gems & XP level-up
        let pickups = update_gems(
            &mut self.gems,
            self.player.pos,
            self.player.magnet_radius,
            self.player.greed_mult,
            self.combo,
            dt,
        );
        if pickups.xp_gained > 0.0 || pickups.shards_gained > 0 {
            self.xp += pickups.xp_gained;
            self.xp_popup_accum += pickups.xp_gained;
            self.run_shards = self.run_shards.saturating_add(pickups.shards_gained);
            self.shard_popup_accum = self.shard_popup_accum.saturating_add(pickups.shards_gained);

            if self.xp >= self.next_xp {
                self.xp -= self.next_xp;
                self.level = self.level.saturating_add(1);
                self.next_xp = next_level_xp(self.next_xp);
                self.sounds
                    .play(&self.sounds.lvlup, self.settings.sound, 0.8);
                self.effects.spawn_ring(
                    self.player.pos,
                    150.0,
                    Color::from_rgba(125, 249, 255, 255),
                    0.4,
                );
                self.roll_levelup();
                return;
            }
        }

        // Batched crisp floating +XP / +SHARD popups
        if self.pickup_popup_timer > 0.0 {
            self.pickup_popup_timer = (self.pickup_popup_timer - dt).max(0.0);
        }
        if self.pickup_popup_timer <= 0.0
            && (self.shard_popup_accum > 0 || self.xp_popup_accum >= 6.0)
        {
            let ppos = self.player.pos;
            if self.shard_popup_accum > 0 {
                let amt = self.shard_popup_accum;
                self.shard_popup_accum = 0;
                let mut tb = TextBuf::<32>::new();
                let _ = write!(&mut tb, "+{} SHARD", amt);
                self.effects.spawn_text(
                    ppos + vec2(self.rng.range_f32(-12.0, 12.0), -30.0),
                    tb,
                    Color::from_rgba(249, 200, 14, 255),
                );
            }
            if self.xp_popup_accum >= 6.0 {
                let amt = self.xp_popup_accum.round() as u32;
                self.xp_popup_accum = 0.0;
                let mut tb = TextBuf::<32>::new();
                let _ = write!(&mut tb, "+{} XP", amt);
                self.effects.spawn_text(
                    ppos + vec2(self.rng.range_f32(-14.0, 14.0), -18.0),
                    tb,
                    Color::from_rgba(125, 249, 255, 255),
                );
            }
            self.pickup_popup_timer = 0.16;
        }

        // Update VFX and record rolling performance metrics
        self.effects.update(dt);
        self.metrics.record_frame(
            dt,
            self.enemies.len(),
            self.bullets.len(),
            self.e_bullets.len(),
            self.effects.particles.len(),
            self.grid.occupied_cells(),
        );
    }

    pub fn trigger_tesla(&mut self) {
        let mut cur = self.player.pos;
        let mut hit = [0usize; 16];
        let mut hit_len = 0_usize;
        let jumps = ((2 + self.player.tesla_lvl) as usize).min(hit.len());
        let dmg = self.player.bullet_dmg * (0.8 + self.player.tesla_lvl as f32 * 0.25);

        for _ in 0..jumps {
            if let Some(idx) = self.find_nearest_enemy_spatial(cur, 260.0, &hit[..hit_len]) {
                hit[hit_len] = idx;
                hit_len += 1;
                let target_pos = self.enemies[idx].pos;
                self.effects.spawn_arc(cur, target_pos);
                self.damage_enemy(idx, dmg, target_pos, true);
                cur = target_pos;
            } else {
                break;
            }
        }
    }

    pub fn trigger_nova(&mut self) {
        let rad =
            145.0 + self.player.nova_lvl as f32 * 28.0 + self.meta.supernova_lvl as f32 * 14.0;
        let ppos = self.player.pos;
        self.effects
            .spawn_ring(ppos, rad, Color::from_rgba(184, 41, 234, 255), 0.35);

        if self.meta.supernova_lvl > 0 {
            let clear_r = rad + 45.0;
            let clear_r2 = clear_r * clear_r;
            self.e_bullets
                .retain(|eb| eb.pos.distance_squared(ppos) > clear_r2);
            self.effects
                .spawn_ring(ppos, clear_r, Color::from_rgba(125, 249, 255, 255), 0.28);
        }

        let dmg = self.player.bullet_dmg
            * (1.1 + self.player.nova_lvl as f32 * 0.35 + self.meta.supernova_lvl as f32 * 0.12);
        let mut targets = [0usize; 96];
        let mut target_len = 0_usize;

        self.grid.for_each_in_aabb(ppos, rad + 40.0, |i| {
            let e = &self.enemies[i];
            if e.alive {
                let rr = rad + e.radius;
                if e.pos.distance_squared(ppos) < rr * rr && target_len < targets.len() {
                    targets[target_len] = i;
                    target_len += 1;
                }
            }
            true
        });

        for &i in &targets[..target_len] {
            if !self.enemies[i].alive {
                continue;
            }
            let diff = self.enemies[i].pos - ppos;
            let d2 = diff.length_squared();
            if d2 > 0.01 {
                self.enemies[i].pos =
                    clamp_to_arena(self.enemies[i].pos + (diff / d2.sqrt()) * 95.0, 20.0);
            }
            let epos = self.enemies[i].pos;
            self.damage_enemy(i, dmg, epos, true);
        }
    }

    pub fn damage_enemy(&mut self, idx: usize, raw_dmg: f32, hit_pos: Vec2, show_num: bool) {
        if idx >= self.enemies.len() || !self.enemies[idx].alive {
            return;
        }
        let roll = roll_hit_damage(raw_dmg, self.player.crit_chance, show_num, &mut self.rng);
        if roll.damage <= 0.0 {
            return;
        }

        self.enemies[idx].hp -= roll.damage;
        self.enemies[idx].flash = 0.075;
        if show_num && self.settings.dmg_nums {
            let col = if roll.is_crit || roll.damage >= 35.0 {
                Color::from_rgba(249, 200, 14, 255)
            } else {
                WHITE
            };
            let pos = self.enemies[idx].pos
                + vec2(self.rng.range_f32(-8.0, 8.0), -self.enemies[idx].radius);
            let mut tb = TextBuf::<32>::new();
            if roll.is_crit {
                let _ = write!(&mut tb, "{}!", roll.damage.ceil() as i32);
            } else {
                let _ = write!(&mut tb, "{}", roll.damage.ceil() as i32);
            }
            self.effects.spawn_text(pos, tb, col);
        }

        if self.enemies[idx].hp <= 0.0 && self.enemies[idx].alive {
            self.enemies[idx].alive = false;
            self.enemies_dirty = true;
            self.kills = self.kills.saturating_add(1);
            self.combo = self.combo.saturating_add(1);
            self.combo_timer = 3.2;

            let kind = self.enemies[idx].kind;
            let elite = self.enemies[idx].elite;
            let epos = self.enemies[idx].pos;
            let erad = self.enemies[idx].radius;
            let e_angle = self.enemies[idx].angle;

            self.sounds.play(
                &self.sounds.boom,
                self.settings.sound,
                if kind == EnemyKind::Boss {
                    1.0
                } else if elite {
                    0.8
                } else {
                    0.6
                },
            );
            self.effects.add_shake(
                if kind == EnemyKind::Boss {
                    12.0
                } else if elite {
                    6.5
                } else {
                    3.5
                },
                self.settings.shake,
            );
            if kind == EnemyKind::Boss {
                self.effects.add_hit_stop(0.055);
            } else if elite {
                self.effects.add_hit_stop(0.024);
            }

            let col = kind.base_color(false);
            self.effects
                .spawn_ring(epos, erad * if elite { 3.2 } else { 2.6 }, col, 0.26);

            let count = if kind == EnemyKind::Boss {
                28
            } else if elite {
                15
            } else {
                9
            };
            for _ in 0..count {
                let a = self.rng.range_f32(0.0, std::f32::consts::TAU);
                let spd = self.rng.range_f32(70.0, 290.0);
                self.effects.spawn_particle(
                    epos,
                    Vec2::from_angle(a) * spd,
                    self.rng.range_f32(0.25, 0.5),
                    3.5,
                    col,
                );
            }

            if kind == EnemyKind::Boss {
                for _ in 0..4 {
                    let off = vec2(
                        self.rng.range_f32(-26.0, 26.0),
                        self.rng.range_f32(-26.0, 26.0),
                    );
                    self.spawn_enemy_at(EnemyKind::Swarmer, epos + off, false);
                }
                if self.meta.overdrive_lvl > 0 {
                    let od = 3.5 + self.meta.overdrive_lvl as f32 * 0.45;
                    self.player.overdrive_timer = self.player.overdrive_timer.max(od);
                    self.effects.spawn_text(
                        epos + vec2(0.0, -24.0),
                        TextBuf::from_str_trunc("BOSS OVERDRIVE!"),
                        Color::from_rgba(249, 200, 14, 255),
                    );
                }
            } else if kind == EnemyKind::Bomber {
                let pellets = if elite { 8 } else { 6 };
                let b_spd = 230.0 + (self.wave as f32 * 8.0).min(110.0);
                let b_dmg = 11.0
                    * (1.0 + self.wave.saturating_sub(1) as f32 * 0.14)
                    * self.difficulty().dmg_mult;
                for p in 0..pellets {
                    let a = e_angle + (p as f32 * std::f32::consts::TAU) / pellets as f32;
                    self.spawn_e_bullet(epos, Vec2::from_angle(a) * b_spd, b_dmg, 4.8);
                }
            }

            let reward = roll_kill_reward(kind, elite, &mut self.rng);
            self.spawn_gem(epos, reward.xp, reward.shard);
        } else {
            self.sounds
                .play(&self.sounds.hit, self.settings.sound, 0.35);
            self.effects.spawn_particle(
                hit_pos,
                vec2(
                    self.rng.range_f32(-70.0, 70.0),
                    self.rng.range_f32(-70.0, 70.0),
                ),
                0.14,
                2.5,
                WHITE,
            );
        }
    }

    pub fn damage_player(&mut self, amount: f32, shake_amt: f32) -> bool {
        let out = apply_player_damage(&mut self.player, amount);
        if out.shield_absorbed >= 3.0 {
            self.effects.spawn_ring(
                self.player.pos,
                34.0,
                Color::from_rgba(125, 249, 255, 255),
                0.18,
            );
        }
        if out.hp_lost <= 0.0 {
            if out.shield_absorbed > 0.0 {
                self.effects
                    .add_shake(shake_amt * 0.45, self.settings.shake);
            }
            return false;
        }

        self.effects.hurt_flash = (self.effects.hurt_flash + 0.45).min(1.0);
        self.effects.add_shake(shake_amt, self.settings.shake);
        if out.lethal {
            self.finish_run(false);
            return true;
        }
        false
    }

    pub fn spawn_e_bullet(&mut self, pos: Vec2, vel: Vec2, dmg: f32, radius: f32) {
        if self.e_bullets.len() >= MAX_EBULLETS {
            return;
        }
        self.e_bullets.push(EnemyBullet {
            pos,
            vel,
            dmg,
            life: 3.6,
            radius,
        });
    }

    pub fn spawn_mine(&mut self, pos: Vec2, dmg: f32, radius: f32) {
        if self.mines.len() >= MAX_MINES {
            self.detonate_mine(0);
        }
        self.mines.push(Mine {
            pos,
            life: 9.0,
            dmg,
            radius,
        });
    }

    pub fn detonate_mine(&mut self, idx: usize) {
        if idx >= self.mines.len() {
            return;
        }
        let m = self.mines.swap_remove(idx);
        let purple = Color::from_rgba(184, 41, 234, 255);
        self.effects.spawn_ring(m.pos, m.radius, purple, 0.34);
        self.sounds
            .play(&self.sounds.boom, self.settings.sound, 0.7);
        for _ in 0..12 {
            let a = self.rng.range_f32(0.0, std::f32::consts::TAU);
            let spd = self.rng.range_f32(80.0, 260.0);
            self.effects
                .spawn_particle(m.pos, Vec2::from_angle(a) * spd, 0.32, 3.5, purple);
        }

        let mut targets = [0usize; 64];
        let mut target_len = 0_usize;
        self.grid.for_each_in_aabb(m.pos, m.radius + 42.0, |i| {
            let e = &self.enemies[i];
            if e.alive {
                let rr = m.radius + e.radius;
                if e.pos.distance_squared(m.pos) <= rr * rr && target_len < targets.len() {
                    targets[target_len] = i;
                    target_len += 1;
                }
            }
            true
        });
        for &i in &targets[..target_len] {
            self.damage_enemy(i, m.dmg, m.pos, true);
        }

        let clear_r2 = (m.radius + 18.0) * (m.radius + 18.0);
        self.e_bullets
            .retain(|eb| eb.pos.distance_squared(m.pos) > clear_r2);
    }

    pub fn spawn_enemy(&mut self, kind: EnemyKind, vp: Viewport) {
        let pos = compute_spawn_position(self.player.pos, vp, &mut self.rng);
        self.spawn_enemy_at(kind, pos, true);
    }

    pub fn spawn_enemy_at(&mut self, kind: EnemyKind, pos: Vec2, allow_elite: bool) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let diff = self.difficulty();
        let enemy = create_enemy(kind, pos, self.wave, diff, allow_elite, &mut self.rng);
        self.enemies.push(enemy);
    }

    pub fn spawn_gem(&mut self, pos: Vec2, xp: f32, shard: u32) {
        if self.gems.len() >= MAX_GEMS {
            return;
        }
        self.gems.push(Gem { pos, xp, shard });
    }
}
