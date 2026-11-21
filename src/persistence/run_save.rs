//! Atomic mid-run persistence (`neon_core_run.dat`) with FNV-1a integrity checksum,
//! `.tmp` -> `.bak` -> rename writes, and `.corrupt` quarantine on parse/checksum failure.

#[cfg(any(not(target_arch = "wasm32"), test))]
use super::migration::fnv1a_64;
use crate::combat::{Bullet, EnemyBullet, Mine};
use crate::enemies::Enemy;
#[cfg(any(not(target_arch = "wasm32"), test))]
use crate::enemies::EnemyKind;
use crate::game::config::{
    MAX_BULLETS, MAX_EBULLETS, MAX_ENEMIES, MAX_GEMS, MAX_HAZARDS, MAX_MINES,
};
use crate::player::{PlayerState, SHIPS};
use crate::progression::{Gem, WaveEvent, DIFFICULTIES};
use crate::world::{clamp_to_arena, BeaconObjective, HazardZone};
use macroquad::math::vec2;
#[cfg(any(not(target_arch = "wasm32"), test))]
use std::fmt::Write as FmtWrite;
#[cfg(any(not(target_arch = "wasm32"), test))]
use std::path::{Path, PathBuf};

#[cfg(any(not(target_arch = "wasm32"), test))]
pub const RUN_SAVE_HEADER_V1: &str = "NEON_CORE_RUN_V1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunSummary {
    pub ship_id: u32,
    pub difficulty: u32,
    pub wave: u32,
    pub level: u32,
    pub kills: u32,
}

#[derive(Clone, Debug)]
pub struct RunSnapshot {
    pub ship_id: u32,
    pub difficulty: u32,
    pub wave: u32,
    pub wave_timer: f32,
    pub wave_event: WaveEvent,
    pub rng_state: u64,

    pub xp: f32,
    pub next_xp: f32,
    pub level: u32,
    pub combo: u32,
    pub combo_timer: f32,
    pub spawn_accum: f32,
    pub run_shards: u32,
    pub kills: u32,
    pub hazard_spawn_timer: f32,
    pub beacon_spawn_timer: f32,
    pub beacon: BeaconObjective,

    pub player: PlayerState,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub e_bullets: Vec<EnemyBullet>,
    pub hazards: Vec<HazardZone>,
    pub mines: Vec<Mine>,
    pub gems: Vec<Gem>,
}

#[cfg(any(not(target_arch = "wasm32"), test))]
#[derive(Debug, PartialEq, Eq)]
pub enum RunParseError {
    Empty,
    InvalidHeader,
    InvalidSyntax,
    MissingChecksum,
    CorruptChecksum,
    InvalidValue,
}

impl RunSnapshot {
    #[inline]
    pub fn summary(&self) -> RunSummary {
        RunSummary {
            ship_id: self.ship_id,
            difficulty: self.difficulty,
            wave: self.wave,
            level: self.level,
            kills: self.kills,
        }
    }

    pub fn validated(mut self) -> Self {
        self.ship_id = self.ship_id.min((SHIPS.len() - 1) as u32);
        self.difficulty = self.difficulty.min((DIFFICULTIES.len() - 1) as u32);
        self.wave = self.wave.max(1);
        self.wave_timer = sanitize_f32(self.wave_timer, 0.0, 0.0, 60.0);
        self.xp = sanitize_f32(self.xp, 0.0, 0.0, 1e9);
        self.next_xp = sanitize_f32(self.next_xp, 25.0, 10.0, 1e9);
        self.level = self.level.max(1);
        self.combo_timer = sanitize_f32(self.combo_timer, 0.0, 0.0, 10.0);
        self.spawn_accum = sanitize_f32(self.spawn_accum, 0.0, 0.0, 20.0);
        self.hazard_spawn_timer = sanitize_f32(self.hazard_spawn_timer, 3.0, 0.0, 30.0);
        self.beacon_spawn_timer = sanitize_f32(self.beacon_spawn_timer, 14.0, 0.0, 60.0);

        self.player.pos = clamp_to_arena(self.player.pos, 18.0);
        self.player.vel = vec2(
            sanitize_f32(self.player.vel.x, 0.0, -3000.0, 3000.0),
            sanitize_f32(self.player.vel.y, 0.0, -3000.0, 3000.0),
        );
        self.player.max_hp = sanitize_f32(self.player.max_hp, 100.0, 10.0, 1e6);
        self.player.hp = sanitize_f32(self.player.hp, self.player.max_hp, 1.0, self.player.max_hp);
        self.player.max_shield = sanitize_f32(self.player.max_shield, 0.0, 0.0, 1e6);
        self.player.shield = sanitize_f32(
            self.player.shield,
            0.0,
            0.0,
            self.player.max_shield.max(0.0),
        );
        self.player.shield_delay = sanitize_f32(self.player.shield_delay, 0.0, 0.0, 20.0);
        self.player.dash_duration = sanitize_f32(self.player.dash_duration, 0.18, 0.05, 1.0);
        self.player.dash_timer =
            sanitize_f32(self.player.dash_timer, 0.0, 0.0, self.player.dash_duration);
        self.player.dash_max_cd = sanitize_f32(self.player.dash_max_cd, 1.6, 0.35, 10.0);
        self.player.dash_cd = sanitize_f32(self.player.dash_cd, 0.0, 0.0, self.player.dash_max_cd);
        self.player.fire_interval = sanitize_f32(self.player.fire_interval, 0.21, 0.04, 5.0);
        self.player.fire_cd = sanitize_f32(self.player.fire_cd, 0.0, 0.0, 5.0);
        self.player.bullet_dmg = sanitize_f32(self.player.bullet_dmg, 18.0, 1.0, 1e6);
        self.player.bullet_count = self.player.bullet_count.clamp(1, 24);
        self.player.bullet_pierce = self.player.bullet_pierce.clamp(1, 24);
        self.player.crit_chance = sanitize_f32(self.player.crit_chance, 0.0, 0.0, 0.95);
        self.player.magnet_radius = sanitize_f32(self.player.magnet_radius, 130.0, 40.0, 2400.0);
        self.player.orbit_count = self.player.orbit_count.clamp(0, 16);
        self.player.tesla_lvl = self.player.tesla_lvl.clamp(0, 20);
        self.player.nova_lvl = self.player.nova_lvl.clamp(0, 20);
        self.player.overdrive_timer = sanitize_f32(self.player.overdrive_timer, 0.0, 0.0, 60.0);

        self.beacon.pos = clamp_to_arena(self.beacon.pos, 100.0);
        self.beacon.radius = sanitize_f32(self.beacon.radius, 82.0, 20.0, 300.0);
        self.beacon.duration = sanitize_f32(self.beacon.duration, 2.4, 0.5, 20.0);
        self.beacon.progress = sanitize_f32(self.beacon.progress, 0.0, 0.0, self.beacon.duration);
        self.beacon.ttl = sanitize_f32(self.beacon.ttl, 0.0, 0.0, 120.0);

        self.enemies.truncate(MAX_ENEMIES);
        self.bullets.truncate(MAX_BULLETS);
        self.e_bullets.truncate(MAX_EBULLETS);
        self.hazards.truncate(MAX_HAZARDS);
        self.mines.truncate(MAX_MINES);
        self.gems.truncate(MAX_GEMS);
        self
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn default_run_save_path() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            let dir = Path::new("/data/data/dev.vex.neon_core/files");
            let _ = std::fs::create_dir_all(dir);
            return dir.join("neon_core_run.dat");
        }
        #[cfg(not(target_os = "android"))]
        {
            if Path::new("/data/data/dev.vex.neon_core").exists() {
                let dir = Path::new("/data/data/dev.vex.neon_core/files");
                let _ = std::fs::create_dir_all(dir);
                dir.join("neon_core_run.dat")
            } else {
                PathBuf::from("neon_core_run.dat")
            }
        }
    }

    pub fn load_default() -> Option<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_from_path(&Self::default_run_save_path())
        }
    }

    pub fn load_summary_default() -> Option<RunSummary> {
        Self::load_default().map(|s| s.summary())
    }

    pub fn save_default(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.save_to_path(&Self::default_run_save_path());
        }
    }

    pub fn delete_default() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            delete_run_save_at(&Self::default_run_save_path());
        }
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn load_from_path(path: &Path) -> Option<Self> {
        if let Ok(raw) = std::fs::read_to_string(path) {
            match parse_run_v1(&raw) {
                Ok(snap) => return Some(snap),
                Err(_) => {
                    let corrupt_path = with_suffix(path, ".corrupt");
                    if !corrupt_path.exists() {
                        let _ = std::fs::copy(path, &corrupt_path);
                    }
                }
            }
        }

        let bak_path = with_suffix(path, ".bak");
        if let Ok(raw_bak) = std::fs::read_to_string(&bak_path) {
            if let Ok(recovered) = parse_run_v1(&raw_bak) {
                let _ = recovered.save_to_path(path);
                return Some(recovered);
            }
        }

        None
    }

    /// Writes atomically via `<path>.tmp` -> `<path>` while preserving previous valid save in `<path>.bak`.
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        let valid = self.clone().validated();
        let encoded = serialize_run_v1(&valid);

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let tmp_path = with_suffix(path, ".tmp");
        let bak_path = with_suffix(path, ".bak");

        {
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp_path)?;
            file.write_all(encoded.as_bytes())?;
            file.sync_all()?;
        }

        if path.exists() {
            if let Ok(prev) = std::fs::read_to_string(path) {
                if parse_run_v1(&prev).is_ok() {
                    let _ = std::fs::copy(path, &bak_path);
                }
            }
        }

        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

#[cfg(any(not(target_arch = "wasm32"), test))]
pub fn delete_run_save_at(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(with_suffix(path, ".bak"));
    let _ = std::fs::remove_file(with_suffix(path, ".tmp"));
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

#[inline]
fn sanitize_f32(v: f32, fallback: f32, min: f32, max: f32) -> f32 {
    if v.is_finite() {
        v.clamp(min, max)
    } else {
        fallback
    }
}

/// Serializes `RunSnapshot` to human-readable `NEON_CORE_RUN_V1` key=value format with FNV-1a checksum.
#[cfg(any(not(target_arch = "wasm32"), test))]
pub fn serialize_run_v1(snap: &RunSnapshot) -> String {
    let mut body = String::with_capacity(2048);
    let p = snap.player;
    let b = snap.beacon;

    let _ = writeln!(body, "ship_id={}", snap.ship_id);
    let _ = writeln!(body, "difficulty={}", snap.difficulty);
    let _ = writeln!(body, "wave={}", snap.wave);
    let _ = writeln!(body, "wave_timer={}", snap.wave_timer);
    let _ = writeln!(body, "wave_event={}", snap.wave_event.as_u8());
    let _ = writeln!(body, "rng_state={}", snap.rng_state);
    let _ = writeln!(body, "xp={}", snap.xp);
    let _ = writeln!(body, "next_xp={}", snap.next_xp);
    let _ = writeln!(body, "level={}", snap.level);
    let _ = writeln!(body, "combo={}", snap.combo);
    let _ = writeln!(body, "combo_timer={}", snap.combo_timer);
    let _ = writeln!(body, "spawn_accum={}", snap.spawn_accum);
    let _ = writeln!(body, "run_shards={}", snap.run_shards);
    let _ = writeln!(body, "kills={}", snap.kills);
    let _ = writeln!(body, "hazard_spawn_timer={}", snap.hazard_spawn_timer);
    let _ = writeln!(body, "beacon_spawn_timer={}", snap.beacon_spawn_timer);
    let _ = writeln!(
        body,
        "beacon={},{},{},{},{},{},{}",
        u8::from(b.active),
        b.pos.x,
        b.pos.y,
        b.radius,
        b.progress,
        b.duration,
        b.ttl
    );

    let _ = writeln!(body, "p_pos={},{}", p.pos.x, p.pos.y);
    let _ = writeln!(body, "p_vel={},{}", p.vel.x, p.vel.y);
    let _ = writeln!(body, "p_aim_angle={}", p.aim_angle);
    let _ = writeln!(body, "p_thrust_accum={}", p.thrust_accum);
    let _ = writeln!(
        body,
        "p_dash={},{},{},{},{},{}",
        p.dash_timer, p.dash_duration, p.dash_cd, p.dash_max_cd, p.dash_dir.x, p.dash_dir.y
    );
    let _ = writeln!(
        body,
        "p_vitals={},{},{},{},{},{},{},{}",
        p.hp,
        p.max_hp,
        p.shield,
        p.max_shield,
        p.shield_delay,
        p.hp_regen,
        p.crit_chance,
        p.greed_mult
    );
    let _ = writeln!(
        body,
        "p_weapon={},{},{},{},{},{},{},{}",
        p.move_speed,
        p.fire_cd,
        p.fire_interval,
        p.bullet_dmg,
        p.bullet_count,
        p.bullet_pierce,
        p.bullet_spread,
        p.magnet_radius
    );
    let _ = writeln!(
        body,
        "p_abilities={},{},{},{},{},{},{},{}",
        p.orbit_count,
        p.orbit_angle,
        p.orbit_dmg,
        p.tesla_lvl,
        p.tesla_timer,
        p.nova_lvl,
        p.nova_timer,
        p.overdrive_timer
    );

    for e in snap.enemies.iter().filter(|e| e.alive).take(MAX_ENEMIES) {
        let _ = writeln!(
            body,
            "enemy={},{},{},{},{},{},{},{},{},{},{},{},{}",
            e.kind as u8,
            u8::from(e.elite),
            e.pos.x,
            e.pos.y,
            e.hp,
            e.max_hp,
            e.radius,
            e.speed,
            e.flash,
            e.timer,
            e.shoot_cd,
            e.angle,
            e.spin
        );
    }
    for h in snap.hazards.iter().take(MAX_HAZARDS) {
        let _ = writeln!(
            body,
            "hazard={},{},{},{},{},{}",
            h.pos.x, h.pos.y, h.radius, h.timer, h.max_timer, h.dmg
        );
    }
    for m in snap.mines.iter().take(MAX_MINES) {
        let _ = writeln!(
            body,
            "mine={},{},{},{},{}",
            m.pos.x, m.pos.y, m.life, m.dmg, m.radius
        );
    }
    for g in snap.gems.iter().take(MAX_GEMS) {
        let _ = writeln!(body, "gem={},{},{},{}", g.pos.x, g.pos.y, g.xp, g.shard);
    }
    for eb in snap.e_bullets.iter().take(MAX_EBULLETS) {
        let _ = writeln!(
            body,
            "ebullet={},{},{},{},{},{},{}",
            eb.pos.x, eb.pos.y, eb.vel.x, eb.vel.y, eb.dmg, eb.life, eb.radius
        );
    }
    for bl in snap.bullets.iter().take(MAX_BULLETS) {
        let _ = writeln!(
            body,
            "bullet={},{},{},{},{},{},{}",
            bl.pos.x, bl.pos.y, bl.vel.x, bl.vel.y, bl.dmg, bl.life, bl.pierce
        );
    }

    let csum = fnv1a_64(body.as_bytes());
    format!("{RUN_SAVE_HEADER_V1}\n{body}checksum={csum:016x}\n")
}

/// Parses and validates a `NEON_CORE_RUN_V1` payload, verifying its 64-bit FNV-1a checksum.
#[cfg(any(not(target_arch = "wasm32"), test))]
pub fn parse_run_v1(raw: &str) -> Result<RunSnapshot, RunParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(RunParseError::Empty);
    }
    let mut lines = trimmed.lines();
    let header = lines.next().ok_or(RunParseError::Empty)?.trim();
    if header != RUN_SAVE_HEADER_V1 {
        return Err(RunParseError::InvalidHeader);
    }

    let mut snap = RunSnapshot {
        ship_id: 0,
        difficulty: 0,
        wave: 1,
        wave_timer: 0.0,
        wave_event: WaveEvent::Normal,
        rng_state: 0x9E37_79B9_7F4A_7C15,
        xp: 0.0,
        next_xp: 25.0,
        level: 1,
        combo: 0,
        combo_timer: 0.0,
        spawn_accum: 0.0,
        run_shards: 0,
        kills: 0,
        hazard_spawn_timer: 5.0,
        beacon_spawn_timer: 14.0,
        beacon: BeaconObjective::default(),
        player: PlayerState::default(),
        enemies: Vec::with_capacity(64),
        bullets: Vec::with_capacity(64),
        e_bullets: Vec::with_capacity(64),
        hazards: Vec::with_capacity(16),
        mines: Vec::with_capacity(8),
        gems: Vec::with_capacity(64),
    };

    let mut body = String::with_capacity(raw.len());
    let mut found_checksum: Option<u64> = None;

    for line in lines {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((key, val)) = l.split_once('=') else {
            return Err(RunParseError::InvalidSyntax);
        };
        let key = key.trim();
        let val = val.trim();

        if key == "checksum" {
            let parsed =
                u64::from_str_radix(val, 16).map_err(|_| RunParseError::CorruptChecksum)?;
            found_checksum = Some(parsed);
            continue;
        }

        body.push_str(key);
        body.push('=');
        body.push_str(val);
        body.push('\n');

        match key {
            "ship_id" => snap.ship_id = parse_u32(val)?,
            "difficulty" => snap.difficulty = parse_u32(val)?,
            "wave" => snap.wave = parse_u32(val)?,
            "wave_timer" => snap.wave_timer = parse_f32_finite(val)?,
            "wave_event" => snap.wave_event = WaveEvent::from_u8(parse_u8(val)?),
            "rng_state" => snap.rng_state = parse_u64(val)?,
            "xp" => snap.xp = parse_f32_finite(val)?,
            "next_xp" => snap.next_xp = parse_f32_finite(val)?,
            "level" => snap.level = parse_u32(val)?,
            "combo" => snap.combo = parse_u32(val)?,
            "combo_timer" => snap.combo_timer = parse_f32_finite(val)?,
            "spawn_accum" => snap.spawn_accum = parse_f32_finite(val)?,
            "run_shards" => snap.run_shards = parse_u32(val)?,
            "kills" => snap.kills = parse_u32(val)?,
            "hazard_spawn_timer" => snap.hazard_spawn_timer = parse_f32_finite(val)?,
            "beacon_spawn_timer" => snap.beacon_spawn_timer = parse_f32_finite(val)?,
            "beacon" => {
                let f = split_exact::<7>(val)?;
                snap.beacon = BeaconObjective {
                    active: parse_u8(f[0])? != 0,
                    pos: vec2(parse_f32_finite(f[1])?, parse_f32_finite(f[2])?),
                    radius: parse_f32_finite(f[3])?,
                    progress: parse_f32_finite(f[4])?,
                    duration: parse_f32_finite(f[5])?,
                    ttl: parse_f32_finite(f[6])?,
                };
            }
            "p_pos" => {
                let f = split_exact::<2>(val)?;
                snap.player.pos = vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?);
            }
            "p_vel" => {
                let f = split_exact::<2>(val)?;
                snap.player.vel = vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?);
            }
            "p_aim_angle" => snap.player.aim_angle = parse_f32_finite(val)?,
            "p_thrust_accum" => snap.player.thrust_accum = parse_f32_finite(val)?,
            "p_dash" => {
                let f = split_exact::<6>(val)?;
                snap.player.dash_timer = parse_f32_finite(f[0])?;
                snap.player.dash_duration = parse_f32_finite(f[1])?;
                snap.player.dash_cd = parse_f32_finite(f[2])?;
                snap.player.dash_max_cd = parse_f32_finite(f[3])?;
                snap.player.dash_dir = vec2(parse_f32_finite(f[4])?, parse_f32_finite(f[5])?);
            }
            "p_vitals" => {
                let f = split_exact::<8>(val)?;
                snap.player.hp = parse_f32_finite(f[0])?;
                snap.player.max_hp = parse_f32_finite(f[1])?;
                snap.player.shield = parse_f32_finite(f[2])?;
                snap.player.max_shield = parse_f32_finite(f[3])?;
                snap.player.shield_delay = parse_f32_finite(f[4])?;
                snap.player.hp_regen = parse_f32_finite(f[5])?;
                snap.player.crit_chance = parse_f32_finite(f[6])?;
                snap.player.greed_mult = parse_f32_finite(f[7])?;
            }
            "p_weapon" => {
                let f = split_exact::<8>(val)?;
                snap.player.move_speed = parse_f32_finite(f[0])?;
                snap.player.fire_cd = parse_f32_finite(f[1])?;
                snap.player.fire_interval = parse_f32_finite(f[2])?;
                snap.player.bullet_dmg = parse_f32_finite(f[3])?;
                snap.player.bullet_count = parse_i32(f[4])?;
                snap.player.bullet_pierce = parse_i32(f[5])?;
                snap.player.bullet_spread = parse_f32_finite(f[6])?;
                snap.player.magnet_radius = parse_f32_finite(f[7])?;
            }
            "p_abilities" => {
                let f = split_exact::<8>(val)?;
                snap.player.orbit_count = parse_i32(f[0])?;
                snap.player.orbit_angle = parse_f32_finite(f[1])?;
                snap.player.orbit_dmg = parse_f32_finite(f[2])?;
                snap.player.tesla_lvl = parse_i32(f[3])?;
                snap.player.tesla_timer = parse_f32_finite(f[4])?;
                snap.player.nova_lvl = parse_i32(f[5])?;
                snap.player.nova_timer = parse_f32_finite(f[6])?;
                snap.player.overdrive_timer = parse_f32_finite(f[7])?;
            }
            "enemy" => {
                if snap.enemies.len() < MAX_ENEMIES {
                    let f = split_exact::<13>(val)?;
                    snap.enemies.push(Enemy {
                        alive: true,
                        kind: EnemyKind::from_u8(parse_u8(f[0])?),
                        elite: parse_u8(f[1])? != 0,
                        pos: clamp_to_arena(
                            vec2(parse_f32_finite(f[2])?, parse_f32_finite(f[3])?),
                            16.0,
                        ),
                        hp: parse_f32_finite(f[4])?.max(0.1),
                        max_hp: parse_f32_finite(f[5])?.max(0.1),
                        radius: parse_f32_finite(f[6])?.clamp(4.0, 90.0),
                        speed: parse_f32_finite(f[7])?.clamp(10.0, 1200.0),
                        flash: parse_f32_finite(f[8])?.clamp(0.0, 1.0),
                        timer: parse_f32_finite(f[9])?.clamp(0.0, 10000.0),
                        shoot_cd: parse_f32_finite(f[10])?.clamp(0.0, 20.0),
                        angle: parse_f32_finite(f[11])?,
                        spin: parse_f32_finite(f[12])?.clamp(-20.0, 20.0),
                    });
                }
            }
            "hazard" => {
                if snap.hazards.len() < MAX_HAZARDS {
                    let f = split_exact::<6>(val)?;
                    let max_t = parse_f32_finite(f[4])?.max(0.1);
                    snap.hazards.push(HazardZone {
                        pos: clamp_to_arena(
                            vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?),
                            30.0,
                        ),
                        radius: parse_f32_finite(f[2])?.clamp(10.0, 300.0),
                        timer: parse_f32_finite(f[3])?.clamp(0.01, max_t),
                        max_timer: max_t,
                        dmg: parse_f32_finite(f[5])?.clamp(1.0, 1e5),
                    });
                }
            }
            "mine" => {
                if snap.mines.len() < MAX_MINES {
                    let f = split_exact::<5>(val)?;
                    snap.mines.push(Mine {
                        pos: clamp_to_arena(
                            vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?),
                            20.0,
                        ),
                        life: parse_f32_finite(f[2])?.clamp(0.01, 60.0),
                        dmg: parse_f32_finite(f[3])?.clamp(1.0, 1e6),
                        radius: parse_f32_finite(f[4])?.clamp(10.0, 400.0),
                    });
                }
            }
            "gem" => {
                if snap.gems.len() < MAX_GEMS {
                    let f = split_exact::<4>(val)?;
                    snap.gems.push(Gem {
                        pos: clamp_to_arena(
                            vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?),
                            10.0,
                        ),
                        xp: parse_f32_finite(f[2])?.clamp(0.0, 1e5),
                        shard: parse_u32(f[3])?,
                    });
                }
            }
            "ebullet" => {
                if snap.e_bullets.len() < MAX_EBULLETS {
                    let f = split_exact::<7>(val)?;
                    snap.e_bullets.push(EnemyBullet {
                        pos: vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?),
                        vel: vec2(parse_f32_finite(f[2])?, parse_f32_finite(f[3])?),
                        dmg: parse_f32_finite(f[4])?.clamp(0.1, 1e5),
                        life: parse_f32_finite(f[5])?.clamp(0.01, 20.0),
                        radius: parse_f32_finite(f[6])?.clamp(1.0, 40.0),
                    });
                }
            }
            "bullet" if snap.bullets.len() < MAX_BULLETS => {
                let f = split_exact::<7>(val)?;
                snap.bullets.push(Bullet {
                    pos: vec2(parse_f32_finite(f[0])?, parse_f32_finite(f[1])?),
                    vel: vec2(parse_f32_finite(f[2])?, parse_f32_finite(f[3])?),
                    dmg: parse_f32_finite(f[4])?.clamp(0.1, 1e6),
                    life: parse_f32_finite(f[5])?.clamp(0.01, 10.0),
                    pierce: parse_i32(f[6])?.clamp(1, 24),
                });
            }
            _ => {
                // Unknown optional future keys are safely skipped.
            }
        }
    }

    let expected = found_checksum.ok_or(RunParseError::MissingChecksum)?;
    let actual = fnv1a_64(body.as_bytes());
    if actual != expected {
        return Err(RunParseError::CorruptChecksum);
    }

    Ok(snap.validated())
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn split_exact<const N: usize>(s: &str) -> Result<[&str; N], RunParseError> {
    let mut out = [""; N];
    let mut count = 0_usize;
    for part in s.split(',') {
        if count >= N {
            return Err(RunParseError::InvalidValue);
        }
        out[count] = part.trim();
        count += 1;
    }
    if count != N {
        return Err(RunParseError::InvalidValue);
    }
    Ok(out)
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_u8(s: &str) -> Result<u8, RunParseError> {
    s.parse().map_err(|_| RunParseError::InvalidValue)
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_u32(s: &str) -> Result<u32, RunParseError> {
    s.parse().map_err(|_| RunParseError::InvalidValue)
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_i32(s: &str) -> Result<i32, RunParseError> {
    s.parse().map_err(|_| RunParseError::InvalidValue)
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_u64(s: &str) -> Result<u64, RunParseError> {
    s.parse().map_err(|_| RunParseError::InvalidValue)
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn parse_f32_finite(s: &str) -> Result<f32, RunParseError> {
    let v: f32 = s.parse().map_err(|_| RunParseError::InvalidValue)?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(RunParseError::InvalidValue)
    }
}
