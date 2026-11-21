use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

const ARENA_RADIUS: f32 = 2800.0;
const DEMO_MAX_WAVE: u32 = 8;

// Spatial hash grid for O(1) enemy lookups
const CELL_SIZE: f32 = 128.0;
const GRID_DIM: usize = 48;
const GRID_HALF: i32 = 24;
const GRID_CELLS: usize = GRID_DIM * GRID_DIM;

const MAX_BULLETS: usize = 520;
const MAX_EBULLETS: usize = 320;
const MAX_HAZARDS: usize = 28;
const MAX_MINES: usize = 18;
const MAX_ENEMIES: usize = 420;
const MAX_PARTICLES: usize = 640;
const MAX_RINGS: usize = 48;
const MAX_TEXTS: usize = 80;
const MAX_GEMS: usize = 380;
const MAX_ARCS: usize = 48;

#[derive(Clone, Copy, PartialEq)]
enum ScreenState {
    Menu,
    Playing,
    Paused,
    Settings,
    LevelUp,
    GameOver { victory: bool },
}

#[derive(Clone, Copy)]
struct Settings {
    sound: bool,
    shake: bool,
    dmg_nums: bool,
    fixed_stick: bool,
    auto_aim: bool,
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
struct MetaSave {
    shards: u32,
    dmg_lvl: u32,
    rate_lvl: u32,
    hp_lvl: u32,
    spd_lvl: u32,
    crit_lvl: u32,
    regen_lvl: u32,
    mag_lvl: u32,
    dash_lvl: u32,
    greed_lvl: u32,
    drone_lvl: u32,
    shield_lvl: u32,
    start_salvo_lvl: u32,
    supernova_lvl: u32,
    overdrive_lvl: u32,
    prestige_lvl: u32,
    selected_ship: u32,
    unlocked_ships_mask: u32,
    difficulty: u32,
    best_wave: u32,
    best_kills: u32,
}

impl Default for MetaSave {
    fn default() -> Self {
        Self {
            shards: 0,
            dmg_lvl: 0,
            rate_lvl: 0,
            hp_lvl: 0,
            spd_lvl: 0,
            crit_lvl: 0,
            regen_lvl: 0,
            mag_lvl: 0,
            dash_lvl: 0,
            greed_lvl: 0,
            drone_lvl: 0,
            shield_lvl: 0,
            start_salvo_lvl: 0,
            supernova_lvl: 0,
            overdrive_lvl: 0,
            prestige_lvl: 0,
            selected_ship: 0,
            unlocked_ships_mask: 1,
            difficulty: 0,
            best_wave: 0,
            best_kills: 0,
        }
    }
}

impl MetaSave {
    fn save_path() -> &'static str {
        if std::path::Path::new("/data/data/dev.vex.neoncore/files").exists() {
            "/data/data/dev.vex.neoncore/files/save.dat"
        } else {
            "neon_core_save.dat"
        }
    }

    fn load() -> Self {
        let mut s = Self::default();
        if let Ok(data) = std::fs::read_to_string(Self::save_path()) {
            let parts: Vec<u32> = data
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();
            if parts.len() >= 5 {
                s.shards = parts[0];
                s.dmg_lvl = parts[1];
                s.rate_lvl = parts[2];
                s.hp_lvl = parts[3];
                s.spd_lvl = parts[4];
                if parts.len() >= 11 {
                    s.crit_lvl = parts[5];
                    s.regen_lvl = parts[6];
                    s.mag_lvl = parts[7];
                    s.dash_lvl = parts[8];
                    s.greed_lvl = parts[9];
                    s.drone_lvl = parts[10];
                }
                if parts.len() >= 21 {
                    s.shield_lvl = parts[11];
                    s.start_salvo_lvl = parts[12];
                    s.supernova_lvl = parts[13];
                    s.overdrive_lvl = parts[14];
                    s.prestige_lvl = parts[15];
                    s.unlocked_ships_mask = (parts[17] | 1).max(1);
                    s.selected_ship = if (parts[16] as usize) < SHIPS.len()
                        && (s.unlocked_ships_mask & (1u32 << parts[16])) != 0
                    {
                        parts[16]
                    } else {
                        0
                    };
                    s.difficulty = parts[18].min((DIFFICULTIES.len() - 1) as u32);
                    s.best_wave = parts[19];
                    s.best_kills = parts[20];
                }
            }
        }
        s
    }

    fn save(&self) {
        let content = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.shards,
            self.dmg_lvl,
            self.rate_lvl,
            self.hp_lvl,
            self.spd_lvl,
            self.crit_lvl,
            self.regen_lvl,
            self.mag_lvl,
            self.dash_lvl,
            self.greed_lvl,
            self.drone_lvl,
            self.shield_lvl,
            self.start_salvo_lvl,
            self.supernova_lvl,
            self.overdrive_lvl,
            self.prestige_lvl,
            self.selected_ship,
            self.unlocked_ships_mask,
            self.difficulty,
            self.best_wave,
            self.best_kills
        );
        let _ = std::fs::write(Self::save_path(), content);
    }

    fn get_lvl(&self, id: u8) -> u32 {
        match id {
            0 => self.dmg_lvl,
            1 => self.rate_lvl,
            2 => self.crit_lvl,
            3 => self.drone_lvl,
            4 => self.dash_lvl,
            5 => self.hp_lvl,
            6 => self.regen_lvl,
            7 => self.spd_lvl,
            8 => self.mag_lvl,
            9 => self.greed_lvl,
            10 => self.shield_lvl,
            11 => self.start_salvo_lvl,
            12 => self.supernova_lvl,
            13 => self.overdrive_lvl,
            _ => self.prestige_lvl,
        }
    }

    fn inc_lvl(&mut self, id: u8) {
        match id {
            0 => self.dmg_lvl += 1,
            1 => self.rate_lvl += 1,
            2 => self.crit_lvl += 1,
            3 => self.drone_lvl += 1,
            4 => self.dash_lvl += 1,
            5 => self.hp_lvl += 1,
            6 => self.regen_lvl += 1,
            7 => self.spd_lvl += 1,
            8 => self.mag_lvl += 1,
            9 => self.greed_lvl += 1,
            10 => self.shield_lvl += 1,
            11 => self.start_salvo_lvl += 1,
            12 => self.supernova_lvl += 1,
            13 => self.overdrive_lvl += 1,
            _ => self.prestige_lvl += 1,
        }
    }
}

#[derive(Clone, Copy)]
struct ShopItem {
    id: u8,
    tab: u8, // 0: BOJOWE, 1: SURVIVAL, 2: ENDGAME
    name: &'static str,
    bonus: &'static str,
    base_cost: u32,
    max_lvl: u32,
    quadratic: bool,
}

#[derive(Clone, Copy)]
struct ShipSpec {
    id: u32,
    name: &'static str,
    short_name: &'static str,
    perk: &'static str,
    cost: u32,
    rgb: [u8; 3],
}

impl ShipSpec {
    fn color(self) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], 255)
    }

    fn color_a(self, a: u8) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], a)
    }
}

#[derive(Clone, Copy)]
struct DifficultySpec {
    id: u32,
    name: &'static str,
    short_tag: &'static str,
    shard_mult: f32,
    hp_mult: f32,
    spd_mult: f32,
    dmg_mult: f32,
    elite_wave: u32,
    double_boss_wave: u32,
    rgb: [u8; 3],
}

impl DifficultySpec {
    fn color(self) -> Color {
        Color::from_rgba(self.rgb[0], self.rgb[1], self.rgb[2], 255)
    }
}

const SHOP_ITEMS: [ShopItem; 15] = [
    ShopItem { id: 0, tab: 0, name: "MOC RDZENIA", bonus: "+15% DMG", base_cost: 15, max_lvl: 15, quadratic: false },
    ShopItem { id: 1, tab: 0, name: "SZYBKOSTRZELNOSC", bonus: "+9% ROF", base_cost: 15, max_lvl: 12, quadratic: false },
    ShopItem { id: 2, tab: 0, name: "KRYTYCZNY RDZEN", bonus: "+8% CRIT 2x", base_cost: 20, max_lvl: 8, quadratic: false },
    ShopItem { id: 3, tab: 0, name: "DRON STARTOWY", bonus: "+1 Ostrze start", base_cost: 40, max_lvl: 4, quadratic: false },
    ShopItem { id: 4, tab: 0, name: "DOPALACZ DASH", bonus: "-12% CD uniku", base_cost: 20, max_lvl: 6, quadratic: false },
    ShopItem { id: 5, tab: 1, name: "PANCERZ KADLUBA", bonus: "+30 Max HP", base_cost: 15, max_lvl: 15, quadratic: false },
    ShopItem { id: 6, tab: 1, name: "REGENERACJA NANO", bonus: "+1.5 HP/s", base_cost: 20, max_lvl: 10, quadratic: false },
    ShopItem { id: 7, tab: 1, name: "SILNIKI MANEWROWE", bonus: "+8% Predkosc", base_cost: 15, max_lvl: 10, quadratic: false },
    ShopItem { id: 8, tab: 1, name: "MAGNES ZBIERANIA", bonus: "+35% Zasieg", base_cost: 15, max_lvl: 8, quadratic: false },
    ShopItem { id: 9, tab: 1, name: "EKSTRAKTOR SHARDOW", bonus: "+20% Shards/XP", base_cost: 25, max_lvl: 8, quadratic: false },
    ShopItem { id: 10, tab: 2, name: "TARCZA PLAZMOWA", bonus: "+35 Max Tarczy + Regen", base_cost: 400, max_lvl: 10, quadratic: true },
    ShopItem { id: 11, tab: 2, name: "OVERCLOCK SALWY", bonus: "+1 Salwa start (+Przebicie)", base_cost: 600, max_lvl: 5, quadratic: true },
    ShopItem { id: 12, tab: 2, name: "PODWOJNA SUPERNOVA", bonus: "EMP niszczy pociski + Start", base_cost: 450, max_lvl: 8, quadratic: true },
    ShopItem { id: 13, tab: 2, name: "REAKTOR OVERDRIVE", bonus: "+1.2s, +12% DMG, Boss OD", base_cost: 350, max_lvl: 8, quadratic: true },
    ShopItem { id: 14, tab: 2, name: "RDZEN PRESTIZU", bonus: "+10% DMG/+8% HP/+10% Shards", base_cost: 500, max_lvl: 25, quadratic: true },
];

const SHIPS: [ShipSpec; 4] = [
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

const DIFFICULTIES: [DifficultySpec; 3] = [
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

#[derive(Clone, Copy)]
struct Bullet {
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    life: f32,
    pierce: i32,
}

#[derive(Clone, Copy)]
struct EnemyBullet {
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    life: f32,
    radius: f32,
}

#[derive(Clone, Copy)]
struct Mine {
    pos: Vec2,
    life: f32,
    dmg: f32,
    radius: f32,
}

#[derive(Clone, Copy)]
struct HazardZone {
    pos: Vec2,
    radius: f32,
    timer: f32,
    max_timer: f32,
    dmg: f32,
}

#[derive(Clone, Copy)]
struct BeaconObjective {
    active: bool,
    pos: Vec2,
    radius: f32,
    progress: f32,
    duration: f32,
    ttl: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum WaveEvent {
    Normal,
    Swarm,
    Orbital,
    Sniper,
    Bomber,
    Boss,
}

impl WaveEvent {
    fn tag(self) -> &'static str {
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

#[derive(Clone, Copy)]
struct Enemy {
    alive: bool,
    kind: u8, // 0: Swarmer, 1: Dasher, 2: Tank, 3: Boss, 4: Sniper, 5: Bomber, 6: Vortex
    elite: bool,
    pos: Vec2,
    hp: f32,
    max_hp: f32,
    radius: f32,
    speed: f32,
    flash: f32,
    timer: f32,
    shoot_cd: f32,
    angle: f32,
    spin: f32,
}

#[derive(Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color,
}

#[derive(Clone, Copy)]
struct Ring {
    pos: Vec2,
    radius: f32,
    max_radius: f32,
    life: f32,
    max_life: f32,
    color: Color,
}

#[derive(Clone)]
struct DmgText {
    pos: Vec2,
    val: String,
    life: f32,
    color: Color,
}

#[derive(Clone, Copy)]
struct Gem {
    pos: Vec2,
    xp: f32,
    shard: u32,
}

#[derive(Clone, Copy)]
struct ArcBolt {
    a: Vec2,
    b: Vec2,
    life: f32,
}

#[derive(Clone, Copy)]
struct UpgradeCard {
    id: u8,
    title: &'static str,
    desc: &'static str,
}

const ALL_UPGRADES: [UpgradeCard; 12] = [
    UpgradeCard { id: 0, title: "ROZDZIELACZ LUF", desc: "+1 Pocisk na salwe" },
    UpgradeCard { id: 1, title: "PLAZMA OVERCHARGE", desc: "+25% Obrazen broni" },
    UpgradeCard { id: 2, title: "CEWKI CYKLERA", desc: "+20% Szybkostrzelnosci" },
    UpgradeCard { id: 3, title: "PENETRATOR", desc: "+1 Przebicie wrogow" },
    UpgradeCard { id: 4, title: "PILA ORBITALNA", desc: "+1 Wirujace ostrze" },
    UpgradeCard { id: 5, title: "LUK TESLI", desc: "Blyskawice lancuchowe" },
    UpgradeCard { id: 6, title: "IMPULS EMP", desc: "Radialna fala uderzeniowa" },
    UpgradeCard { id: 7, title: "CHLODZENIE DASH", desc: "-25% Cooldownu uniku" },
    UpgradeCard { id: 8, title: "NANONAPRAWA", desc: "+40% HP i +25 Max HP" },
    UpgradeCard { id: 9, title: "MAGNES RDZENIA", desc: "+45% Zasieg zbierania" },
    UpgradeCard { id: 10, title: "TURBO KRYTYK", desc: "+15% Szansy na CRIT 2x" },
    UpgradeCard { id: 11, title: "AUTONAPRAWA", desc: "+3 HP/s regeneracji" },
];

struct SoundBank {
    shoot: Option<Sound>,
    hit: Option<Sound>,
    boom: Option<Sound>,
    dash: Option<Sound>,
    lvlup: Option<Sound>,
    shoot_cd: f32,
}

fn synth_wav(freq_start: f32, freq_end: f32, duration: f32, volume: f32, square: bool) -> Vec<u8> {
    let sample_rate = 22050u32;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let data_size = (num_samples * 2) as u32;
    let mut buf = Vec::with_capacity(44 + data_size as usize);

    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_size).to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    let mut phase = 0.0f32;
    for i in 0..num_samples {
        let t = i as f32 / num_samples as f32;
        let freq = freq_start * (freq_end / freq_start).powf(t);
        phase += freq / sample_rate as f32;
        let wave = if square {
            if phase.fract() < 0.5 { 1.0 } else { -1.0 }
        } else {
            (phase * std::f32::consts::TAU).sin()
        };
        let env = (1.0 - t).powf(1.6) * volume;
        let sample = (wave * env * 32767.0).clamp(-32767.0, 32767.0) as i16;
        buf.extend_from_slice(&sample.to_le_bytes());
    }
    buf
}

impl SoundBank {
    async fn new() -> Self {
        Self {
            shoot: load_sound_from_bytes(&synth_wav(460.0, 110.0, 0.07, 0.22, true)).await.ok(),
            hit: load_sound_from_bytes(&synth_wav(190.0, 55.0, 0.05, 0.25, false)).await.ok(),
            boom: load_sound_from_bytes(&synth_wav(135.0, 24.0, 0.24, 0.42, true)).await.ok(),
            dash: load_sound_from_bytes(&synth_wav(170.0, 540.0, 0.14, 0.30, false)).await.ok(),
            lvlup: load_sound_from_bytes(&synth_wav(330.0, 880.0, 0.28, 0.32, false)).await.ok(),
            shoot_cd: 0.0,
        }
    }

    fn play(&self, snd: &Option<Sound>, enabled: bool, vol: f32) {
        if !enabled {
            return;
        }
        if let Some(s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: vol,
                },
            );
        }
    }
}

struct Stick {
    touch_id: Option<u64>,
    start: Vec2,
    cur: Vec2,
    dir: Vec2,
}

impl Stick {
    fn new() -> Self {
        Self {
            touch_id: None,
            start: Vec2::ZERO,
            cur: Vec2::ZERO,
            dir: Vec2::ZERO,
        }
    }
}

struct Game {
    state: ScreenState,
    prev_state: ScreenState,
    shop_tab: u8,
    settings: Settings,
    meta: MetaSave,
    sounds: SoundBank,

    bullets: Vec<Bullet>,
    e_bullets: Vec<EnemyBullet>,
    hazards: Vec<HazardZone>,
    mines: Vec<Mine>,
    enemies: Vec<Enemy>,
    cell_head: Vec<i32>,
    enemy_next: Vec<i32>,
    particles: Vec<Particle>,
    rings: Vec<Ring>,
    texts: Vec<DmgText>,
    gems: Vec<Gem>,
    arcs: Vec<ArcBolt>,
    stars: Vec<(Vec2, f32, f32)>,

    player_pos: Vec2,
    player_vel: Vec2,
    aim_angle: f32,
    thrust_accum: f32,

    dash_timer: f32,
    dash_duration: f32,
    dash_cd: f32,
    dash_max_cd: f32,
    dash_dir: Vec2,

    hp: f32,
    max_hp: f32,
    shield: f32,
    max_shield: f32,
    shield_delay: f32,
    hp_regen: f32,
    crit_chance: f32,
    greed_mult: f32,
    xp: f32,
    next_xp: f32,
    level: u32,
    wave: u32,
    wave_timer: f32,
    wave_event: WaveEvent,
    banner_text: String,
    banner_sub: String,
    banner_timer: f32,
    banner_color: Color,
    hazard_spawn_timer: f32,
    beacon_spawn_timer: f32,
    beacon: BeaconObjective,
    overdrive_timer: f32,
    combo: u32,
    combo_timer: f32,
    hurt_flash: f32,
    spawn_accum: f32,
    run_shards: u32,
    kills: u32,
    is_new_record: bool,
    shake: f32,

    move_speed: f32,
    fire_cd: f32,
    fire_interval: f32,
    bullet_dmg: f32,
    bullet_count: i32,
    bullet_pierce: i32,
    bullet_spread: f32,
    magnet_radius: f32,

    orbit_count: i32,
    orbit_angle: f32,
    orbit_dmg: f32,
    tesla_lvl: i32,
    tesla_timer: f32,
    nova_lvl: i32,
    nova_timer: f32,

    left_stick: Stick,
    right_stick: Stick,
    level_choices: [UpgradeCard; 3],
}

#[inline]
fn cell_coord(v: f32) -> usize {
    let c = (v / CELL_SIZE).floor() as i32 + GRID_HALF;
    c.clamp(0, (GRID_DIM - 1) as i32) as usize
}

impl Game {
    fn new(sounds: SoundBank) -> Self {
        let stars = (0..160)
            .map(|_| {
                (
                    vec2(rand::gen_range(-1600.0, 1600.0), rand::gen_range(-1600.0, 1600.0)),
                    if rand::gen_range(0, 10) < 6 { 0.25 } else { 0.55 },
                    rand::gen_range(1.5, 2.6),
                )
            })
            .collect();

        Self {
            state: ScreenState::Menu,
            prev_state: ScreenState::Menu,
            shop_tab: 0,
            settings: Settings::default(),
            meta: MetaSave::load(),
            sounds,

            bullets: Vec::with_capacity(MAX_BULLETS),
            e_bullets: Vec::with_capacity(MAX_EBULLETS),
            hazards: Vec::with_capacity(MAX_HAZARDS),
            mines: Vec::with_capacity(MAX_MINES),
            enemies: Vec::with_capacity(MAX_ENEMIES),
            cell_head: vec![-1; GRID_CELLS],
            enemy_next: vec![-1; MAX_ENEMIES],
            particles: Vec::with_capacity(MAX_PARTICLES),
            rings: Vec::with_capacity(MAX_RINGS),
            texts: Vec::with_capacity(MAX_TEXTS),
            gems: Vec::with_capacity(MAX_GEMS),
            arcs: Vec::with_capacity(MAX_ARCS),
            stars,

            player_pos: Vec2::ZERO,
            player_vel: Vec2::ZERO,
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
            xp: 0.0,
            next_xp: 25.0,
            level: 1,
            wave: 1,
            wave_timer: 0.0,
            wave_event: WaveEvent::Normal,
            banner_text: String::new(),
            banner_sub: String::new(),
            banner_timer: 0.0,
            banner_color: Color::from_rgba(5, 217, 232, 255),
            hazard_spawn_timer: 5.0,
            beacon_spawn_timer: 14.0,
            beacon: BeaconObjective {
                active: false,
                pos: Vec2::ZERO,
                radius: 82.0,
                progress: 0.0,
                duration: 2.4,
                ttl: 0.0,
            },
            overdrive_timer: 0.0,
            combo: 0,
            combo_timer: 0.0,
            hurt_flash: 0.0,
            spawn_accum: 0.0,
            run_shards: 0,
            kills: 0,
            is_new_record: false,
            shake: 0.0,

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

            left_stick: Stick::new(),
            right_stick: Stick::new(),
            level_choices: [ALL_UPGRADES[0], ALL_UPGRADES[1], ALL_UPGRADES[2]],
        }
    }

    fn rebuild_spatial_grid(&mut self) {
        self.cell_head.fill(-1);
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.alive {
                continue;
            }
            let cx = cell_coord(e.pos.x);
            let cy = cell_coord(e.pos.y);
            let cell = cy * GRID_DIM + cx;
            self.enemy_next[i] = self.cell_head[cell];
            self.cell_head[cell] = i as i32;
        }
    }

    fn difficulty(&self) -> DifficultySpec {
        DIFFICULTIES[(self.meta.difficulty as usize).min(DIFFICULTIES.len() - 1)]
    }

    fn selected_ship(&self) -> ShipSpec {
        SHIPS[(self.meta.selected_ship as usize).min(SHIPS.len() - 1)]
    }

    fn start_run(&mut self) {
        self.bullets.clear();
        self.e_bullets.clear();
        self.hazards.clear();
        self.mines.clear();
        self.enemies.clear();
        self.particles.clear();
        self.rings.clear();
        self.texts.clear();
        self.gems.clear();
        self.arcs.clear();

        self.left_stick = Stick::new();
        self.right_stick = Stick::new();

        let ship_id = self.meta.selected_ship;
        let diff = self.difficulty();
        let ship = self.selected_ship();
        let prestige_dmg = 1.0 + self.meta.prestige_lvl as f32 * 0.10;
        let prestige_hp = 1.0 + self.meta.prestige_lvl as f32 * 0.08;
        let prestige_shards = 1.0 + self.meta.prestige_lvl as f32 * 0.10;

        self.player_pos = Vec2::ZERO;
        self.player_vel = Vec2::ZERO;
        self.dash_timer = 0.0;
        self.dash_cd = 0.0;
        self.dash_duration = 0.18 + self.meta.dash_lvl as f32 * 0.015;
        let ship_dash_mult = if ship_id == 3 { 0.68 } else { 1.0 };
        self.dash_max_cd = (1.6 * 0.88f32.powi(self.meta.dash_lvl as i32) * ship_dash_mult).max(0.42);

        self.wave = 1;
        self.wave_timer = 0.0;
        self.wave_event = WaveEvent::Normal;
        self.banner_text = format!("FALA 1 — {} [{}]", ship.name, diff.name);
        self.banner_sub = if diff.id == 0 {
            "ZBIERAJ PRZEKAZNIKI I UNIKAJ OGNIA".to_string()
        } else {
            format!("MNOZNIK SHARDOW {:.1}x — EKSTREMALNE ZAGROZENIE", diff.shard_mult)
        };
        self.banner_timer = 3.2;
        self.banner_color = diff.color();
        self.hazard_spawn_timer = if diff.id == 2 { 3.2 } else { 5.0 };
        self.beacon_spawn_timer = 14.0;
        self.beacon.active = false;
        self.beacon.progress = 0.0;
        self.overdrive_timer = 0.0;
        self.combo = 0;
        self.combo_timer = 0.0;
        self.hurt_flash = 0.0;
        self.spawn_accum = 0.0;
        self.level = 1;
        self.xp = 0.0;
        self.next_xp = 25.0;
        self.run_shards = 0;
        self.kills = 0;
        self.is_new_record = false;

        let ship_hp_mult = if ship_id == 2 { 1.7 } else if ship_id == 1 { 0.85 } else { 1.0 };
        self.max_hp = ((100.0 + self.meta.hp_lvl as f32 * 30.0) * prestige_hp * ship_hp_mult).round();
        self.hp = self.max_hp;
        self.max_shield = self.meta.shield_lvl as f32 * 35.0 + if ship_id == 2 { 60.0 } else { 0.0 };
        self.shield = self.max_shield;
        self.shield_delay = 0.0;
        self.hp_regen = self.meta.regen_lvl as f32 * 1.5;
        self.crit_chance = (self.meta.crit_lvl as f32 * 0.08 + if ship_id == 3 { 0.15 } else { 0.0 }).min(0.90);
        self.greed_mult = (1.0 + self.meta.greed_lvl as f32 * 0.20) * prestige_shards * diff.shard_mult;

        let ship_dmg_mult = if ship_id == 2 { 1.35 } else { 1.0 };
        self.bullet_dmg = 18.0 * (1.0 + self.meta.dmg_lvl as f32 * 0.15) * prestige_dmg * ship_dmg_mult;
        let ship_rof_mult = if ship_id == 1 { 0.78 } else if ship_id == 2 { 1.35 } else { 1.0 };
        self.fire_interval = (0.21 * 0.91f32.powi(self.meta.rate_lvl as i32) * ship_rof_mult).max(0.048);
        let ship_spd_mult = if ship_id == 1 { 1.22 } else if ship_id == 2 { 0.86 } else { 1.0 };
        self.move_speed = 275.0 * (1.0 + self.meta.spd_lvl as f32 * 0.08) * ship_spd_mult;

        let ship_extra_bullets = if ship_id == 2 { 3 } else if ship_id == 1 { 1 } else { 0 };
        self.bullet_count = 1 + self.meta.start_salvo_lvl as i32 + ship_extra_bullets;
        self.bullet_pierce = 1 + (self.meta.start_salvo_lvl / 2) as i32;
        self.bullet_spread = if ship_id == 2 { 0.165 } else { 0.11 };
        self.magnet_radius = 130.0 * (1.0 + self.meta.mag_lvl as f32 * 0.35);

        self.orbit_count = self.meta.drone_lvl as i32;
        self.orbit_dmg = 16.0 + self.meta.drone_lvl as f32 * 4.0;
        self.nova_lvl = ((self.meta.supernova_lvl + 1) / 2) as i32;
        self.tesla_lvl = (self.meta.supernova_lvl / 2) as i32;
        self.nova_timer = 1.6;
        self.tesla_timer = 1.0;

        for _ in 0..(8 + diff.id * 2) {
            self.spawn_enemy(0);
        }
        self.state = ScreenState::Playing;
    }

    fn trigger_dash(&mut self) {
        if self.dash_cd > 0.0 {
            return;
        }
        self.sounds.play(&self.sounds.dash, self.settings.sound, 0.8);
        if self.meta.selected_ship == 3 {
            let ppos = self.player_pos;
            let mdmg = self.bullet_dmg * 4.5;
            self.spawn_mine(ppos, mdmg, 138.0);
        }
        let mut dir = self.left_stick.dir;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { dir.y -= 1.0; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { dir.y += 1.0; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dir.x -= 1.0; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dir.x += 1.0; }

        if dir.length_squared() > 0.01 {
            self.dash_dir = dir.normalize();
        } else {
            self.dash_dir = Vec2::from_angle(self.aim_angle);
        }
        self.dash_timer = self.dash_duration;
        self.dash_cd = self.dash_max_cd;
        let ring_col = self.selected_ship().color();
        self.spawn_ring(self.player_pos, 75.0, ring_col, 0.25);
    }

    fn pause_rect(&self) -> Rect {
        let sw = screen_width();
        let size = if sw < 460.0 { 36.0 } else { 42.0 };
        Rect::new(sw - size - 10.0, 10.0, size, size)
    }

    fn update_playing(&mut self, dt: f32) {
        let sw = screen_width();
        let sh = screen_height();

        let pause_rect = self.pause_rect();
        let dash_center = vec2(sw - 58.0, sh - 62.0);

        for touch in touches() {
            match touch.phase {
                TouchPhase::Started => {
                    if pause_rect.contains(touch.position) {
                        self.state = ScreenState::Paused;
                        return;
                    }
                    if touch.position.distance_squared(dash_center) <= 44.0 * 44.0 {
                        self.trigger_dash();
                        continue;
                    }
                    let want_move = if self.settings.auto_aim {
                        self.left_stick.touch_id.is_none()
                    } else {
                        touch.position.x < sw * 0.5 && self.left_stick.touch_id.is_none()
                    };

                    if want_move {
                        let start = if self.settings.fixed_stick {
                            vec2(86.0, sh - 96.0)
                        } else {
                            touch.position
                        };
                        self.left_stick.touch_id = Some(touch.id);
                        self.left_stick.start = start;
                        self.left_stick.cur = touch.position;
                        let delta = touch.position - start;
                        let len = delta.length();
                        self.left_stick.dir = if len > 2.0 {
                            (delta / len) * (len.min(44.0) / 44.0)
                        } else {
                            Vec2::ZERO
                        };
                    } else if self.right_stick.touch_id.is_none() {
                        self.right_stick.touch_id = Some(touch.id);
                        self.right_stick.start = touch.position;
                        self.right_stick.cur = touch.position;
                        self.right_stick.dir = Vec2::ZERO;
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(touch.id) == self.left_stick.touch_id {
                        self.left_stick.cur = touch.position;
                        let delta = touch.position - self.left_stick.start;
                        let len = delta.length();
                        self.left_stick.dir = if len > 2.0 {
                            (delta / len) * (len.min(44.0) / 44.0)
                        } else {
                            Vec2::ZERO
                        };
                    } else if Some(touch.id) == self.right_stick.touch_id {
                        self.right_stick.cur = touch.position;
                        let delta = touch.position - self.right_stick.start;
                        let len = delta.length();
                        self.right_stick.dir = if len > 4.0 { delta / len } else { Vec2::ZERO };
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(touch.id) == self.left_stick.touch_id {
                        self.left_stick.touch_id = None;
                        self.left_stick.dir = Vec2::ZERO;
                    } else if Some(touch.id) == self.right_stick.touch_id {
                        self.right_stick.touch_id = None;
                        self.right_stick.dir = Vec2::ZERO;
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift) {
            self.trigger_dash();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.state = ScreenState::Paused;
            return;
        }

        let diff = self.difficulty();

        if self.hp_regen > 0.0 && self.hp < self.max_hp {
            self.hp = (self.hp + self.hp_regen * dt).min(self.max_hp);
        }
        if self.max_shield > 0.0 {
            if self.shield_delay > 0.0 {
                self.shield_delay = (self.shield_delay - dt).max(0.0);
            } else if self.shield < self.max_shield {
                let s_regen = 18.0 + self.meta.shield_lvl as f32 * 5.0;
                self.shield = (self.shield + s_regen * dt).min(self.max_shield);
            }
        }
        if self.banner_timer > 0.0 {
            self.banner_timer = (self.banner_timer - dt).max(0.0);
        }
        if self.hurt_flash > 0.0 {
            self.hurt_flash = (self.hurt_flash - dt * 2.8).max(0.0);
        }
        if self.overdrive_timer > 0.0 {
            self.overdrive_timer = (self.overdrive_timer - dt).max(0.0);
        }
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        // Wave progression & dynamic wave events (22s waves)
        self.wave_timer += dt;
        if self.wave_timer >= 22.0 {
            self.wave_timer = 0.0;
            self.wave += 1;
            if cfg!(feature = "demo") && self.wave > DEMO_MAX_WAVE {
                self.finish_run(true);
                return;
            }
            self.start_next_wave();
        }

        // Dynamic Spawner tailored to current wave, active event & difficulty
        let event_rate_mult = match self.wave_event {
            WaveEvent::Swarm => 1.55,
            WaveEvent::Bomber => 1.25,
            _ => 1.0,
        };
        let diff_rate_mult = 1.0 + diff.id as f32 * 0.22;
        let spawn_rate = ((2.0 + self.wave as f32 * 0.72) * event_rate_mult * diff_rate_mult).min(18.0);
        self.spawn_accum += dt * spawn_rate;
        while self.spawn_accum >= 1.0 {
            self.spawn_accum -= 1.0;
            let r = rand::gen_range(0.0f32, 1.0);
            let kind = match self.wave_event {
                WaveEvent::Sniper => {
                    if r > 0.72 { if self.wave >= 5 { 6 } else { 4 } }
                    else if r > 0.36 { 4 }
                    else if r > 0.18 { 1 }
                    else { 0 }
                }
                WaveEvent::Bomber => {
                    if r > 0.56 { 5 }
                    else if r > 0.34 { 1 }
                    else if r > 0.22 { 2 }
                    else { 0 }
                }
                WaveEvent::Swarm => {
                    if r > 0.75 { 1 }
                    else if r > 0.65 && self.wave >= 4 { 5 }
                    else { 0 }
                }
                _ => {
                    if self.wave >= 5 && r > 0.90 { 6 }
                    else if self.wave >= 4 && r > 0.80 { 5 }
                    else if self.wave >= 2 && r > 0.68 { 4 }
                    else if self.wave >= 3 && r > 0.56 { 2 }
                    else if self.wave >= 2 && r > 0.38 { 1 }
                    else { 0 }
                }
            };
            self.spawn_enemy(kind);
        }

        // Orbital Hazard Strikes (Wave 3+ or Wave 2+ on Nightmare)
        if self.wave >= (if diff.id == 2 { 2 } else { 3 }) || self.wave_event == WaveEvent::Orbital {
            self.hazard_spawn_timer -= dt;
            if self.hazard_spawn_timer <= 0.0 {
                let is_orbital = self.wave_event == WaveEvent::Orbital;
                let h_div = 1.0 + diff.id as f32 * 0.16;
                self.hazard_spawn_timer = if is_orbital {
                    ((1.05 - self.wave as f32 * 0.04) / h_div).max(0.36)
                } else {
                    ((3.4 - self.wave as f32 * 0.14) / h_div).max(1.0)
                };
                let count = if is_orbital { if self.wave >= 7 || diff.id == 2 { 3 } else { 2 } } else { 1 };
                for h_idx in 0..count {
                    let lead = if h_idx == 0 { 0.45 } else { rand::gen_range(0.1, 0.85) };
                    let hpos = self.player_pos
                        + self.player_vel * lead
                        + vec2(rand::gen_range(-140.0, 140.0), rand::gen_range(-140.0, 140.0));
                    let hrad = if is_orbital { rand::gen_range(66.0, 86.0) } else { rand::gen_range(58.0, 76.0) };
                    let warn_t = (1.38 - self.wave as f32 * 0.025 - diff.id as f32 * 0.08).max(0.84);
                    let hdmg = (22.0 + self.wave as f32 * 2.8) * diff.dmg_mult;
                    self.spawn_hazard(hpos, hrad, warn_t, hdmg);
                }
            }
        }

        // Interactive Map Objective: Overload Beacon
        if !self.beacon.active {
            self.beacon_spawn_timer -= dt;
            if self.beacon_spawn_timer <= 0.0 {
                self.spawn_beacon();
            }
        } else {
            self.beacon.ttl -= dt;
            let b_r2 = self.beacon.radius * self.beacon.radius;
            if self.player_pos.distance_squared(self.beacon.pos) <= b_r2 {
                self.beacon.progress += dt;
                if self.beacon.progress >= self.beacon.duration {
                    self.complete_beacon();
                    if self.state != ScreenState::Playing {
                        return;
                    }
                }
            } else {
                self.beacon.progress = (self.beacon.progress - dt * 0.45).max(0.0);
            }
            if self.beacon.active && self.beacon.ttl <= 0.0 {
                self.beacon.active = false;
                self.beacon_spawn_timer = 18.0;
            }
        }

        // Player physics
        if self.dash_cd > 0.0 {
            self.dash_cd = (self.dash_cd - dt).max(0.0);
        }

        let trail_col = if self.overdrive_timer > 0.0 {
            Color::from_rgba(249, 200, 14, 255)
        } else {
            Color::from_rgba(5, 217, 232, 255)
        };
        let active_move_speed = if self.overdrive_timer > 0.0 {
            self.move_speed * 1.3
        } else {
            self.move_speed
        };

        if self.dash_timer > 0.0 {
            self.dash_timer -= dt;
            self.player_vel = self.dash_dir * 860.0;
            self.spawn_particle(
                self.player_pos,
                vec2(rand::gen_range(-45.0, 45.0), rand::gen_range(-45.0, 45.0)),
                0.22,
                5.0,
                trail_col,
            );
        } else {
            let mut mv = self.left_stick.dir;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { mv.y -= 1.0; }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { mv.y += 1.0; }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { mv.x -= 1.0; }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { mv.x += 1.0; }
            if mv.length_squared() > 1.0 {
                mv = mv.normalize();
            }

            let target_vel = mv * active_move_speed;
            let accel = 1.0 - (-18.0 * dt).exp();
            self.player_vel += (target_vel - self.player_vel) * accel;

            if self.player_vel.length_squared() > 35.0 * 35.0 {
                self.thrust_accum += dt * 45.0;
                while self.thrust_accum >= 1.0 {
                    self.thrust_accum -= 1.0;
                    let back = self.player_vel.y.atan2(self.player_vel.x)
                        + std::f32::consts::PI
                        + rand::gen_range(-0.25, 0.25);
                    let dir = Vec2::from_angle(back);
                    self.spawn_particle(
                        self.player_pos + dir * 12.0,
                        dir * 120.0,
                        0.2,
                        3.0,
                        trail_col,
                    );
                }
            }
        }

        self.player_pos += self.player_vel * dt;
        let limit = ARENA_RADIUS - 18.0;
        let dist_sq = self.player_pos.length_squared();
        if dist_sq > limit * limit {
            self.player_pos = (self.player_pos / dist_sq.sqrt()) * limit;
        }

        // Build spatial grid once per frame
        self.rebuild_spatial_grid();

        // Aiming
        let manual_aim = self.right_stick.touch_id.is_some() && self.right_stick.dir.length_squared() > 0.01;
        let nearest_pos = self.find_nearest_enemy_spatial(self.player_pos, 620.0, &[]).map(|idx| self.enemies[idx].pos);
        if manual_aim {
            self.aim_angle = self.right_stick.dir.y.atan2(self.right_stick.dir.x);
        } else if self.settings.auto_aim {
            if let Some(target) = nearest_pos {
                let d = target - self.player_pos;
                self.aim_angle = d.y.atan2(d.x);
            } else if self.player_vel.length_squared() > 400.0 {
                self.aim_angle = self.player_vel.y.atan2(self.player_vel.x);
            }
        } else if self.player_vel.length_squared() > 400.0 {
            self.aim_angle = self.player_vel.y.atan2(self.player_vel.x);
        }

        // Primary weapon (Overdrive boosts fire rate & Overdrive Reactor boosts damage)
        self.sounds.shoot_cd = (self.sounds.shoot_cd - dt).max(0.0);
        self.fire_cd -= dt;
        if self.fire_cd <= 0.0 && (manual_aim || !self.settings.auto_aim || nearest_pos.is_some()) {
            self.fire_cd = if self.overdrive_timer > 0.0 {
                self.fire_interval * 0.68
            } else {
                self.fire_interval
            };
            if self.sounds.shoot_cd <= 0.0 {
                self.sounds.play(&self.sounds.shoot, self.settings.sound, 0.5);
                self.sounds.shoot_cd = 0.055;
            }
            let spread = self.bullet_spread;
            let shot_dmg = if self.overdrive_timer > 0.0 {
                self.bullet_dmg * (1.0 + self.meta.overdrive_lvl as f32 * 0.12)
            } else {
                self.bullet_dmg
            };
            for i in 0..self.bullet_count {
                let offset = (i as f32 - (self.bullet_count - 1) as f32 * 0.5) * spread;
                let a = self.aim_angle + offset;
                let dir = Vec2::from_angle(a);
                self.spawn_bullet(self.player_pos + dir * 18.0, dir * 760.0, shot_dmg, self.bullet_pierce);
            }
        }

        // Orbital saws (spatial query)
        if self.orbit_count > 0 {
            self.orbit_angle += dt * 3.6;
            for i in 0..self.orbit_count {
                let a = self.orbit_angle + (i as f32 * std::f32::consts::TAU) / self.orbit_count as f32;
                let bpos = self.player_pos + Vec2::from_angle(a) * 74.0;
                let cx0 = cell_coord(bpos.x - 52.0);
                let cx1 = cell_coord(bpos.x + 52.0);
                let cy0 = cell_coord(bpos.y - 52.0);
                let cy1 = cell_coord(bpos.y + 52.0);
                for cy in cy0..=cy1 {
                    let row = cy * GRID_DIM;
                    for cx in cx0..=cx1 {
                        let mut j = self.cell_head[row + cx];
                        while j != -1 {
                            let idx = j as usize;
                            j = self.enemy_next[idx];
                            if !self.enemies[idx].alive { continue; }
                            let rr = self.enemies[idx].radius + 12.0;
                            if self.enemies[idx].pos.distance_squared(bpos) < rr * rr {
                                self.damage_enemy(idx, self.orbit_dmg * dt * 8.0, bpos, false);
                            }
                        }
                    }
                }
            }
        }

        // Chain Tesla
        if self.tesla_lvl > 0 {
            self.tesla_timer -= dt;
            if self.tesla_timer <= 0.0 {
                self.tesla_timer = (1.4 - self.tesla_lvl as f32 * 0.18).max(0.45);
                self.trigger_tesla();
            }
        }

        // EMP Nova
        if self.nova_lvl > 0 {
            self.nova_timer -= dt;
            if self.nova_timer <= 0.0 {
                self.nova_timer = (4.5 - self.nova_lvl as f32 * 0.45).max(1.8);
                self.trigger_nova();
            }
        }

        // Update bullets (dense swap_remove + spatial grid collision)
        let arena_r2 = ARENA_RADIUS * ARENA_RADIUS;
        let mut bi = self.bullets.len();
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
            let cx0 = cell_coord(bpos.x - 45.0);
            let cx1 = cell_coord(bpos.x + 45.0);
            let cy0 = cell_coord(bpos.y - 45.0);
            let cy1 = cell_coord(bpos.y + 45.0);
            let mut removed = false;
            'cell_loop: for cy in cy0..=cy1 {
                let row = cy * GRID_DIM;
                for cx in cx0..=cx1 {
                    let mut j = self.cell_head[row + cx];
                    while j != -1 {
                        let idx = j as usize;
                        j = self.enemy_next[idx];
                        if !self.enemies[idx].alive { continue; }
                        let rr = self.enemies[idx].radius + 7.0;
                        if self.enemies[idx].pos.distance_squared(bpos) < rr * rr {
                            self.damage_enemy(idx, bdmg, bpos, true);
                            self.bullets[bi].pierce -= 1;
                            if self.bullets[bi].pierce <= 0 {
                                self.bullets.swap_remove(bi);
                                removed = true;
                                break 'cell_loop;
                            }
                        }
                    }
                }
            }
            let _ = removed;
        }

        // Update Phantom Wraith proximity plasma mines
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

        // Compact dead enemies & update living ones (7 archetypes + boss bullet-hell)
        self.enemies.retain(|e| e.alive);
        let e_dmg_scale = (1.0 + (self.wave.saturating_sub(1)) as f32 * 0.14) * diff.dmg_mult;
        for i in 0..self.enemies.len() {
            if self.enemies[i].flash > 0.0 {
                self.enemies[i].flash -= dt;
            }
            let epos = self.enemies[i].pos;
            let kind = self.enemies[i].kind;
            let elite = self.enemies[i].elite;
            let erad = self.enemies[i].radius;
            let to_player = self.player_pos - epos;
            let dist_sq = to_player.length_squared();
            let dist = if dist_sq > 0.000001 { dist_sq.sqrt() } else { 0.001 };
            let dir = to_player / dist;
            self.enemies[i].angle += self.enemies[i].spin * dt;
            self.enemies[i].timer += dt;
            let e_angle = self.enemies[i].angle;
            let e_speed = self.enemies[i].speed;

            if kind == 1 {
                let mult = if (self.enemies[i].timer % 2.1) > 1.55 { 3.25 } else { 0.65 };
                self.enemies[i].pos += dir * e_speed * mult * dt;
            } else if kind == 4 {
                // Sniper Drone: kites & strafes, telegraphs red laser, fires aimed crimson plasma
                let strafe_dir = if (i & 1) == 0 { 1.0 } else { -1.0 };
                let radial = if dist > 360.0 { 1.0 } else if dist < 245.0 { -0.85 } else { 0.05 };
                let mv = vec2(
                    dir.x * radial - dir.y * 0.72 * strafe_dir,
                    dir.y * radial + dir.x * 0.72 * strafe_dir,
                );
                self.enemies[i].pos += mv * e_speed * dt;

                self.enemies[i].shoot_cd -= dt;
                if self.enemies[i].shoot_cd <= 0.0 && dist < 680.0 {
                    self.enemies[i].shoot_cd = (2.35 - self.wave as f32 * 0.06).max(1.35);
                    let base_a = to_player.y.atan2(to_player.x);
                    let shot_spd = 460.0 + (self.wave as f32 * 14.0).min(180.0);
                    let shot_dmg = (if elite { 18.0 } else { 13.0 }) * e_dmg_scale;
                    if elite {
                        for s in -1..=1 {
                            let a = base_a + s as f32 * 0.16;
                            self.spawn_e_bullet(epos, Vec2::from_angle(a) * shot_spd, shot_dmg, 5.5);
                        }
                    } else {
                        self.spawn_e_bullet(epos, Vec2::from_angle(base_a) * shot_spd, shot_dmg, 5.0);
                    }
                    self.sounds.play(&self.sounds.shoot, self.settings.sound, 0.35);
                }
            } else if kind == 5 {
                // Shrapnel Bomber: rushes & explodes into radial shrapnel
                self.enemies[i].pos += dir * e_speed * dt;
                if dist < 68.0 {
                    self.enemies[i].alive = false;
                    self.spawn_ring(epos, 115.0, Color::from_rgba(170, 255, 45, 255), 0.32);
                    self.sounds.play(&self.sounds.boom, self.settings.sound, 0.75);
                    let pellets = if elite { 10 } else { 7 };
                    let b_spd = 250.0 + (self.wave as f32 * 8.0).min(120.0);
                    let b_dmg = 13.0 * e_dmg_scale;
                    for p in 0..pellets {
                        let a = e_angle + (p as f32 * std::f32::consts::TAU) / pellets as f32;
                        self.spawn_e_bullet(epos, Vec2::from_angle(a) * b_spd, b_dmg, 5.0);
                    }
                    if self.dash_timer <= 0.0 && dist < 82.0 {
                        if self.damage_player(20.0 * e_dmg_scale, 9.0) {
                            return;
                        }
                    }
                    continue;
                }
            } else if kind == 6 {
                // Vortex Elite: orbits, gently pulls player inward, fires spiral volleys
                let strafe_dir = if (i & 1) == 0 { 1.0 } else { -1.0 };
                let radial = if dist > 290.0 { 0.9 } else if dist < 195.0 { -0.6 } else { 0.0 };
                let mv = vec2(
                    dir.x * radial - dir.y * 0.85 * strafe_dir,
                    dir.y * radial + dir.x * 0.85 * strafe_dir,
                );
                self.enemies[i].pos += mv * e_speed * dt;

                if dist < 270.0 && self.dash_timer <= 0.0 {
                    let pull = (1.0 - dist / 270.0) * (if elite { 115.0 } else { 80.0 }) * dt;
                    self.player_pos -= dir * pull;
                }
                self.enemies[i].shoot_cd -= dt;
                if self.enemies[i].shoot_cd <= 0.0 && dist < 640.0 {
                    self.enemies[i].shoot_cd = (2.5 - self.wave as f32 * 0.06).max(1.45);
                    let count = if elite { 6 } else { 4 };
                    let b_spd = 220.0 + (self.wave as f32 * 8.0).min(110.0);
                    let b_dmg = 12.0 * e_dmg_scale;
                    for s in 0..count {
                        let a = e_angle + (s as f32 * std::f32::consts::TAU) / count as f32;
                        self.spawn_e_bullet(epos, Vec2::from_angle(a) * b_spd, b_dmg, 5.2);
                    }
                }
            } else if kind == 3 {
                // Overlord Boss: bullet-hell rings + enraged phase below 50% HP
                let enraged = self.enemies[i].hp < self.enemies[i].max_hp * 0.5;
                let step = e_speed * if enraged { 1.32 } else { 1.0 } * dt;
                self.enemies[i].pos += dir * step;

                self.enemies[i].shoot_cd -= dt;
                if self.enemies[i].shoot_cd <= 0.0 && dist < 760.0 {
                    self.enemies[i].shoot_cd = if enraged {
                        (1.35 - self.wave as f32 * 0.025).max(0.85)
                    } else {
                        (2.0 - self.wave as f32 * 0.03).max(1.25)
                    };
                    let ring_cnt = ((if enraged { 12 } else { 8 }) + (self.wave / 3)).min(16);
                    let b_spd = (if enraged { 275.0 } else { 225.0 }) + (self.wave as f32 * 9.0).min(120.0);
                    let b_dmg = 15.0 * e_dmg_scale;
                    for s in 0..ring_cnt {
                        let a = e_angle + (s as f32 * std::f32::consts::TAU) / ring_cnt as f32;
                        self.spawn_e_bullet(epos, Vec2::from_angle(a) * b_spd, b_dmg, 6.0);
                    }
                    if enraged {
                        let aim_a = to_player.y.atan2(to_player.x);
                        self.spawn_e_bullet(epos, Vec2::from_angle(aim_a) * (b_spd + 140.0), b_dmg * 1.15, 6.5);
                    }
                    let rcol = if enraged {
                        Color::from_rgba(255, 58, 92, 255)
                    } else {
                        Color::from_rgba(184, 41, 234, 255)
                    };
                    self.spawn_ring(epos, erad * 2.1, rcol, 0.28);
                }
            } else {
                self.enemies[i].pos += dir * e_speed * dt;
            }

            if self.dash_timer <= 0.0 && dist < erad + 14.0 {
                let base_dps = if kind == 3 { 78.0 } else if kind == 2 { 52.0 } else { 42.0 };
                let dps = base_dps * if elite { 1.3 } else { 1.0 };
                if self.damage_player(dps * dt, 5.0) {
                    return;
                }
            }
        }
        self.enemies.retain(|e| e.alive);

        // Update enemy bullets (dense swap_remove + Dash i-frame dodging)
        let mut ebi = self.e_bullets.len();
        while ebi > 0 {
            ebi -= 1;
            let vel = self.e_bullets[ebi].vel;
            self.e_bullets[ebi].pos += vel * dt;
            self.e_bullets[ebi].life -= dt;
            let pos = self.e_bullets[ebi].pos;
            if self.e_bullets[ebi].life <= 0.0 || pos.length_squared() > arena_r2 {
                self.e_bullets.swap_remove(ebi);
                continue;
            }
            if self.dash_timer <= 0.0 {
                let hit_r = self.e_bullets[ebi].radius + 10.5;
                if pos.distance_squared(self.player_pos) < hit_r * hit_r {
                    let dmg = self.e_bullets[ebi].dmg;
                    self.e_bullets.swap_remove(ebi);
                    self.spawn_particle(
                        self.player_pos,
                        vec2(rand::gen_range(-90.0, 90.0), rand::gen_range(-90.0, 90.0)),
                        0.22,
                        4.0,
                        Color::from_rgba(255, 58, 92, 255),
                    );
                    if self.damage_player(dmg, 7.0) {
                        return;
                    }
                }
            }
        }

        // Update Orbital Hazard Strikes
        let mut hi = self.hazards.len();
        while hi > 0 {
            hi -= 1;
            self.hazards[hi].timer -= dt;
            if self.hazards[hi].timer <= 0.0 {
                let h = self.hazards.swap_remove(hi);
                let crimson = Color::from_rgba(255, 58, 92, 255);
                self.spawn_ring(h.pos, h.radius * 1.15, crimson, 0.34);
                self.sounds.play(&self.sounds.boom, self.settings.sound, 0.55);
                for _ in 0..10 {
                    let a = rand::gen_range(0.0, std::f32::consts::TAU);
                    let spd = rand::gen_range(60.0, 210.0);
                    self.spawn_particle(h.pos, Vec2::from_angle(a) * spd, rand::gen_range(0.2, 0.42), 3.5, crimson);
                }
                if self.dash_timer <= 0.0 {
                    let hr = h.radius + 8.0;
                    if self.player_pos.distance_squared(h.pos) <= hr * hr {
                        if self.damage_player(h.dmg, 10.0) {
                            return;
                        }
                    }
                }
            }
        }

        // Update gems (dense swap_remove + combo XP multiplier)
        let mag_r2 = self.magnet_radius * self.magnet_radius;
        let combo_mult = 1.0 + (self.combo as f32 * 0.025).min(1.0);
        let mut gi = self.gems.len();
        while gi > 0 {
            gi -= 1;
            let to_p = self.player_pos - self.gems[gi].pos;
            let dist_sq = to_p.length_squared();
            if dist_sq < mag_r2 && dist_sq > 0.01 {
                self.gems[gi].pos += (to_p / dist_sq.sqrt()) * 510.0 * dt;
            }
            if dist_sq < 24.0 * 24.0 {
                let g = self.gems.swap_remove(gi);
                self.xp += g.xp * self.greed_mult * combo_mult;
                if g.shard > 0 {
                    self.run_shards += ((g.shard as f32 * self.greed_mult).round() as u32).max(1);
                }
                if self.xp >= self.next_xp {
                    self.xp -= self.next_xp;
                    self.level += 1;
                    self.next_xp = (self.next_xp * 1.32 + 12.0).floor();
                    self.sounds.play(&self.sounds.lvlup, self.settings.sound, 0.8);
                    self.spawn_ring(self.player_pos, 150.0, Color::from_rgba(125, 249, 255, 255), 0.4);
                    self.roll_levelup();
                    return;
                }
            }
        }

        // Update VFX (dense swap_remove)
        let mut pi = self.particles.len();
        while pi > 0 {
            pi -= 1;
            let vel = self.particles[pi].vel;
            self.particles[pi].pos += vel * dt;
            self.particles[pi].life -= dt;
            if self.particles[pi].life <= 0.0 {
                self.particles.swap_remove(pi);
            }
        }
        let mut ri = self.rings.len();
        while ri > 0 {
            ri -= 1;
            self.rings[ri].life -= dt;
            self.rings[ri].radius = self.rings[ri].max_radius * (1.0 - (self.rings[ri].life / self.rings[ri].max_life).max(0.0));
            if self.rings[ri].life <= 0.0 {
                self.rings.swap_remove(ri);
            }
        }
        let mut ti = self.texts.len();
        while ti > 0 {
            ti -= 1;
            self.texts[ti].pos.y -= 52.0 * dt;
            self.texts[ti].life -= dt;
            if self.texts[ti].life <= 0.0 {
                self.texts.swap_remove(ti);
            }
        }
        let mut ai = self.arcs.len();
        while ai > 0 {
            ai -= 1;
            self.arcs[ai].life -= dt;
            if self.arcs[ai].life <= 0.0 {
                self.arcs.swap_remove(ai);
            }
        }

        if self.shake > 0.0 {
            self.shake = (self.shake - dt * 30.0).max(0.0);
        }
    }

    fn trigger_tesla(&mut self) {
        let mut cur = self.player_pos;
        let mut hit = Vec::with_capacity(8);
        let jumps = 2 + self.tesla_lvl;
        for _ in 0..jumps {
            if let Some(idx) = self.find_nearest_enemy_spatial(cur, 260.0, &hit) {
                hit.push(idx);
                let target_pos = self.enemies[idx].pos;
                self.spawn_arc(cur, target_pos);
                let dmg = self.bullet_dmg * (0.8 + self.tesla_lvl as f32 * 0.25);
                self.damage_enemy(idx, dmg, target_pos, true);
                cur = target_pos;
            } else {
                break;
            }
        }
    }

    fn trigger_nova(&mut self) {
        let rad = 145.0 + self.nova_lvl as f32 * 28.0 + self.meta.supernova_lvl as f32 * 14.0;
        let ppos = self.player_pos;
        self.spawn_ring(ppos, rad, Color::from_rgba(184, 41, 234, 255), 0.35);

        if self.meta.supernova_lvl > 0 {
            let clear_r = rad + 45.0;
            let clear_r2 = clear_r * clear_r;
            self.e_bullets.retain(|eb| eb.pos.distance_squared(ppos) > clear_r2);
            self.spawn_ring(ppos, clear_r, Color::from_rgba(125, 249, 255, 255), 0.28);
        }

        let span = rad + 40.0;
        let cx0 = cell_coord(ppos.x - span);
        let cx1 = cell_coord(ppos.x + span);
        let cy0 = cell_coord(ppos.y - span);
        let cy1 = cell_coord(ppos.y + span);
        for cy in cy0..=cy1 {
            let row = cy * GRID_DIM;
            for cx in cx0..=cx1 {
                let mut j = self.cell_head[row + cx];
                while j != -1 {
                    let i = j as usize;
                    j = self.enemy_next[i];
                    if !self.enemies[i].alive { continue; }
                    let diff = self.enemies[i].pos - ppos;
                    let rr = rad + self.enemies[i].radius;
                    let d2 = diff.length_squared();
                    if d2 < rr * rr {
                        if d2 > 0.01 {
                            self.enemies[i].pos += (diff / d2.sqrt()) * 95.0;
                        }
                        let epos = self.enemies[i].pos;
                        let dmg = self.bullet_dmg
                            * (1.1 + self.nova_lvl as f32 * 0.35 + self.meta.supernova_lvl as f32 * 0.12);
                        self.damage_enemy(i, dmg, epos, true);
                    }
                }
            }
        }
    }

    fn damage_enemy(&mut self, idx: usize, raw_dmg: f32, hit_pos: Vec2, show_num: bool) {
        let is_crit = show_num && self.crit_chance > 0.0 && rand::gen_range(0.0f32, 1.0) < self.crit_chance;
        let dmg = if is_crit { raw_dmg * 2.0 } else { raw_dmg };

        self.enemies[idx].hp -= dmg;
        self.enemies[idx].flash = 0.06;
        if show_num && self.settings.dmg_nums {
            let col = if is_crit || dmg >= 35.0 { Color::from_rgba(249, 200, 14, 255) } else { WHITE };
            let pos = self.enemies[idx].pos + vec2(rand::gen_range(-8.0, 8.0), -self.enemies[idx].radius);
            let txt = if is_crit { format!("{}!", dmg.ceil() as i32) } else { format!("{}", dmg.ceil() as i32) };
            self.spawn_text(pos, txt, col);
        }

        if self.enemies[idx].hp <= 0.0 && self.enemies[idx].alive {
            self.enemies[idx].alive = false;
            self.kills += 1;
            self.combo += 1;
            self.combo_timer = 3.2;
            let kind = self.enemies[idx].kind;
            let elite = self.enemies[idx].elite;
            let epos = self.enemies[idx].pos;
            let erad = self.enemies[idx].radius;
            let e_angle = self.enemies[idx].angle;

            self.sounds.play(&self.sounds.boom, self.settings.sound, if kind == 3 { 1.0 } else if elite { 0.8 } else { 0.6 });
            if self.settings.shake {
                self.shake = self.shake.max(if kind == 3 { 12.0 } else if elite { 6.5 } else { 3.5 });
            }

            let col = match kind {
                1 => Color::from_rgba(255, 123, 0, 255),
                2 => Color::from_rgba(249, 200, 14, 255),
                3 => Color::from_rgba(184, 41, 234, 255),
                4 => Color::from_rgba(255, 58, 92, 255),
                5 => Color::from_rgba(170, 255, 45, 255),
                6 => Color::from_rgba(125, 110, 255, 255),
                _ => Color::from_rgba(255, 42, 109, 255),
            };
            self.spawn_ring(epos, erad * if elite { 3.2 } else { 2.6 }, col, 0.26);

            let count = if kind == 3 { 28 } else if elite { 15 } else { 9 };
            for _ in 0..count {
                let a = rand::gen_range(0.0, std::f32::consts::TAU);
                let spd = rand::gen_range(70.0, 290.0);
                self.spawn_particle(epos, Vec2::from_angle(a) * spd, rand::gen_range(0.25, 0.5), 3.5, col);
            }

            if kind == 3 {
                for _ in 0..4 {
                    let off = vec2(rand::gen_range(-26.0, 26.0), rand::gen_range(-26.0, 26.0));
                    self.spawn_enemy_at(0, epos + off, false);
                }
                if self.meta.overdrive_lvl > 0 {
                    let od = 3.5 + self.meta.overdrive_lvl as f32 * 0.45;
                    self.overdrive_timer = self.overdrive_timer.max(od);
                    self.spawn_text(
                        epos + vec2(0.0, -24.0),
                        "BOSS OVERDRIVE!".to_string(),
                        Color::from_rgba(249, 200, 14, 255),
                    );
                }
            } else if kind == 5 {
                let pellets = if elite { 8 } else { 6 };
                let b_spd = 230.0 + (self.wave as f32 * 8.0).min(110.0);
                let b_dmg = 11.0 * (1.0 + (self.wave.saturating_sub(1)) as f32 * 0.14) * self.difficulty().dmg_mult;
                for p in 0..pellets {
                    let a = e_angle + (p as f32 * std::f32::consts::TAU) / pellets as f32;
                    self.spawn_e_bullet(epos, Vec2::from_angle(a) * b_spd, b_dmg, 4.8);
                }
            }

            let mut shard = match kind {
                3 => 14,
                6 => 5,
                2 | 4 | 5 => 3,
                _ => if rand::gen_range(0, 100) < 32 { 1 } else { 0 },
            };
            let mut xp = match kind {
                3 => 55.0,
                6 => 24.0,
                2 => 16.0,
                4 | 5 => 14.0,
                _ => 6.0,
            };
            if elite {
                shard = (shard * 3).max(3);
                xp = (xp * 2.4).round();
            }
            self.spawn_gem(epos, xp, shard);
        } else {
            self.sounds.play(&self.sounds.hit, self.settings.sound, 0.35);
            self.spawn_particle(
                hit_pos,
                vec2(rand::gen_range(-70.0, 70.0), rand::gen_range(-70.0, 70.0)),
                0.14,
                2.5,
                WHITE,
            );
        }
    }

    fn roll_levelup(&mut self) {
        let mut pool = ALL_UPGRADES;
        for i in (1..pool.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            pool.swap(i, j);
        }
        self.level_choices = [pool[0], pool[1], pool[2]];
        self.state = ScreenState::LevelUp;
    }

    fn apply_upgrade(&mut self, id: u8) {
        match id {
            0 => self.bullet_count += 1,
            1 => self.bullet_dmg *= 1.25,
            2 => self.fire_interval = (self.fire_interval * 0.83).max(0.045),
            3 => self.bullet_pierce += 1,
            4 => { self.orbit_count += 1; self.orbit_dmg += 6.0; }
            5 => self.tesla_lvl += 1,
            6 => self.nova_lvl += 1,
            7 => self.dash_max_cd = (self.dash_max_cd * 0.75).max(0.45),
            8 => { self.max_hp += 25.0; self.hp = (self.hp + self.max_hp * 0.4).min(self.max_hp); }
            9 => self.magnet_radius *= 1.45,
            10 => self.crit_chance = (self.crit_chance + 0.15).min(0.95),
            11 => self.hp_regen += 3.0,
            _ => {}
        }
        self.state = ScreenState::Playing;
    }

    fn finish_run(&mut self, victory: bool) {
        self.meta.shards += self.run_shards;
        self.is_new_record = self.wave > self.meta.best_wave || self.kills > self.meta.best_kills;
        if self.wave > self.meta.best_wave {
            self.meta.best_wave = self.wave;
        }
        if self.kills > self.meta.best_kills {
            self.meta.best_kills = self.kills;
        }
        self.meta.save();
        self.state = ScreenState::GameOver { victory };
    }

    fn find_nearest_enemy_spatial(&self, origin: Vec2, max_range: f32, exclude: &[usize]) -> Option<usize> {
        let mut best = None;
        let mut best_d2 = max_range * max_range;
        let cx0 = cell_coord(origin.x - max_range);
        let cx1 = cell_coord(origin.x + max_range);
        let cy0 = cell_coord(origin.y - max_range);
        let cy1 = cell_coord(origin.y + max_range);

        for cy in cy0..=cy1 {
            let row = cy * GRID_DIM;
            for cx in cx0..=cx1 {
                let mut j = self.cell_head[row + cx];
                while j != -1 {
                    let idx = j as usize;
                    j = self.enemy_next[idx];
                    let e = &self.enemies[idx];
                    if !e.alive || exclude.contains(&idx) {
                        continue;
                    }
                    let d2 = e.pos.distance_squared(origin);
                    if d2 < best_d2 {
                        best_d2 = d2;
                        best = Some(idx);
                    }
                }
            }
        }
        best
    }

    fn spawn_bullet(&mut self, pos: Vec2, vel: Vec2, dmg: f32, pierce: i32) {
        if self.bullets.len() >= MAX_BULLETS { return; }
        self.bullets.push(Bullet { pos, vel, dmg, life: 1.15, pierce });
    }

    fn damage_player(&mut self, amount: f32, shake_amt: f32) -> bool {
        let mut rem = amount;
        if self.max_shield > 0.0 {
            self.shield_delay = 3.8;
            if self.shield > 0.0 {
                let absorbed = self.shield.min(rem);
                self.shield -= absorbed;
                rem -= absorbed;
                if absorbed >= 3.0 {
                    let ppos = self.player_pos;
                    self.spawn_ring(ppos, 34.0, Color::from_rgba(125, 249, 255, 255), 0.18);
                }
            }
        }
        if rem <= 0.0 {
            if self.settings.shake {
                self.shake = self.shake.max(shake_amt * 0.45);
            }
            return false;
        }

        self.hp -= rem;
        self.hurt_flash = (self.hurt_flash + 0.45).min(1.0);
        if self.settings.shake {
            self.shake = self.shake.max(shake_amt);
        }
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.finish_run(false);
            return true;
        }
        false
    }

    fn start_next_wave(&mut self) {
        let w = self.wave;
        let diff = self.difficulty();
        if w % 3 == 0 {
            self.wave_event = WaveEvent::Boss;
            let boss_cnt = if w >= diff.double_boss_wave + 9 {
                4
            } else if w >= diff.double_boss_wave + 6 {
                3
            } else if w >= diff.double_boss_wave {
                2
            } else {
                1
            };
            self.banner_text = format!("FALA {} — ALARM: OVERLORD BOSS x{}!", w, boss_cnt);
            self.banner_sub = if boss_cnt >= 2 {
                "WIELU BOSSOW + ESKORTA SNAJPEROW".to_string()
            } else {
                "UNIKAJ FAL POCISKOW — DASH DAJE NIESMIERTELNOSC".to_string()
            };
            self.banner_color = Color::from_rgba(184, 41, 234, 255);
            for _ in 0..boss_cnt {
                self.spawn_enemy(3);
            }
            for _ in 0..(2 + w / 4 + diff.id) {
                self.spawn_enemy(4);
            }
        } else {
            let ev = if w >= 4 {
                match w % 4 {
                    0 => WaveEvent::Swarm,
                    1 => WaveEvent::Orbital,
                    2 => WaveEvent::Sniper,
                    _ => WaveEvent::Bomber,
                }
            } else {
                match w % 3 {
                    0 => WaveEvent::Sniper,
                    1 => WaveEvent::Orbital,
                    _ => WaveEvent::Swarm,
                }
            };
            self.wave_event = ev;
            match ev {
                WaveEvent::Swarm => {
                    self.banner_text = format!("FALA {} — ROJ ZASADZKI!", w);
                    self.banner_sub = "OTACZAJA CIE SZYBKIE JEDNOSTKI".to_string();
                    self.banner_color = Color::from_rgba(255, 123, 0, 255);
                    let ring_cnt = (9 + w).min(20);
                    let lim = ARENA_RADIUS - 50.0;
                    for k in 0..ring_cnt {
                        let a = (k as f32 * std::f32::consts::TAU) / ring_cnt as f32;
                        let mut pos = self.player_pos + Vec2::from_angle(a) * 430.0;
                        let d2 = pos.length_squared();
                        if d2 > lim * lim {
                            pos = (pos / d2.sqrt()) * lim;
                        }
                        self.spawn_enemy_at(if k % 3 == 0 { 1 } else { 0 }, pos, true);
                    }
                }
                WaveEvent::Orbital => {
                    self.banner_text = format!("FALA {} — OSTRZAL ORBITALNY!", w);
                    self.banner_sub = "UCIEKAJ Z CZERWONYCH STREF UDERZENIA".to_string();
                    self.banner_color = Color::from_rgba(255, 58, 92, 255);
                    self.hazard_spawn_timer = 0.55;
                }
                WaveEvent::Sniper => {
                    self.banner_text = format!("FALA {} — ESKADRA SNAJPEROW!", w);
                    self.banner_sub = "UWAGA NA LASERY NAMIERZAJACE I WIRY".to_string();
                    self.banner_color = Color::from_rgba(125, 110, 255, 255);
                    for k in 0..(4 + w / 2) {
                        self.spawn_enemy(if k % 3 == 0 && w >= 5 { 6 } else { 4 });
                    }
                }
                _ => {
                    self.banner_text = format!("FALA {} — FALA BOMBOWCOW!", w);
                    self.banner_sub = "NISZCZ Z DYSTANSU — EKSPLODUJA ODLAMKAMI".to_string();
                    self.banner_color = Color::from_rgba(170, 255, 45, 255);
                    for _ in 0..(5 + w / 2) {
                        self.spawn_enemy(5);
                    }
                }
            }
        }
        self.banner_timer = 3.6;
        self.sounds.play(&self.sounds.lvlup, self.settings.sound, 0.55);
    }

    fn spawn_beacon(&mut self) {
        let a = rand::gen_range(0.0, std::f32::consts::TAU);
        let dist = rand::gen_range(540.0, 1020.0);
        let mut pos = self.player_pos + Vec2::from_angle(a) * dist;
        let lim = ARENA_RADIUS - 260.0;
        let d2 = pos.length_squared();
        if d2 > lim * lim {
            pos = (pos / d2.sqrt()) * lim;
        }
        self.beacon = BeaconObjective {
            active: true,
            pos,
            radius: 82.0,
            progress: 0.0,
            duration: 2.4,
            ttl: 28.0,
        };
        if self.banner_timer <= 0.4 {
            self.banner_text = "PRZEKAZNIK MOCY AKTYWNY".to_string();
            self.banner_sub = "PRZEJMIJ ZLOTY PUNKT NA RADARZE — OVERDRIVE + SHARDS".to_string();
            self.banner_color = Color::from_rgba(249, 200, 14, 255);
            self.banner_timer = 3.0;
        }
    }

    fn complete_beacon(&mut self) {
        let bpos = self.beacon.pos;
        self.beacon.active = false;
        self.beacon.progress = 0.0;
        self.beacon_spawn_timer = 22.0;
        let od_dur = 5.5 + self.meta.overdrive_lvl as f32 * 1.2;
        self.overdrive_timer = self.overdrive_timer.max(od_dur);

        let bonus_shards = ((6 + self.wave * 2) as f32 * self.greed_mult).round() as u32;
        let bonus_xp = (22.0 + self.wave as f32 * 6.0) * self.greed_mult;
        self.run_shards += bonus_shards;
        self.xp += bonus_xp;

        let clear_r2 = 480.0 * 480.0;
        self.e_bullets.retain(|eb| eb.pos.distance_squared(bpos) > clear_r2);

        let gold = Color::from_rgba(249, 200, 14, 255);
        self.spawn_ring(bpos, 360.0, gold, 0.5);
        self.spawn_text(bpos + vec2(0.0, -28.0), format!("OVERDRIVE! +{} SHARDS", bonus_shards), gold);
        self.sounds.play(&self.sounds.lvlup, self.settings.sound, 0.95);
        if self.settings.shake {
            self.shake = self.shake.max(7.0);
        }
        if self.xp >= self.next_xp {
            self.xp -= self.next_xp;
            self.level += 1;
            self.next_xp = (self.next_xp * 1.32 + 12.0).floor();
            self.roll_levelup();
        }
    }

    fn spawn_e_bullet(&mut self, pos: Vec2, vel: Vec2, dmg: f32, radius: f32) {
        if self.e_bullets.len() >= MAX_EBULLETS { return; }
        self.e_bullets.push(EnemyBullet { pos, vel, dmg, life: 3.6, radius });
    }

    fn spawn_hazard(&mut self, mut pos: Vec2, radius: f32, max_timer: f32, dmg: f32) {
        if self.hazards.len() >= MAX_HAZARDS { return; }
        let lim = ARENA_RADIUS - radius - 20.0;
        let d2 = pos.length_squared();
        if d2 > lim * lim {
            pos = (pos / d2.sqrt()) * lim;
        }
        self.hazards.push(HazardZone { pos, radius, timer: max_timer, max_timer, dmg });
    }

    fn spawn_mine(&mut self, pos: Vec2, dmg: f32, radius: f32) {
        if self.mines.len() >= MAX_MINES {
            self.detonate_mine(0);
        }
        self.mines.push(Mine { pos, life: 9.0, dmg, radius });
    }

    fn detonate_mine(&mut self, idx: usize) {
        let m = self.mines.swap_remove(idx);
        let purple = Color::from_rgba(184, 41, 234, 255);
        self.spawn_ring(m.pos, m.radius, purple, 0.34);
        self.sounds.play(&self.sounds.boom, self.settings.sound, 0.7);
        for _ in 0..12 {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let spd = rand::gen_range(80.0, 260.0);
            self.spawn_particle(m.pos, Vec2::from_angle(a) * spd, 0.32, 3.5, purple);
        }

        let span = m.radius + 42.0;
        let cx0 = cell_coord(m.pos.x - span);
        let cx1 = cell_coord(m.pos.x + span);
        let cy0 = cell_coord(m.pos.y - span);
        let cy1 = cell_coord(m.pos.y + span);
        for cy in cy0..=cy1 {
            let row = cy * GRID_DIM;
            for cx in cx0..=cx1 {
                let mut j = self.cell_head[row + cx];
                while j != -1 {
                    let i = j as usize;
                    j = self.enemy_next[i];
                    if !self.enemies[i].alive { continue; }
                    let rr = m.radius + self.enemies[i].radius;
                    if self.enemies[i].pos.distance_squared(m.pos) <= rr * rr {
                        self.damage_enemy(i, m.dmg, m.pos, true);
                    }
                }
            }
        }

        let clear_r2 = (m.radius + 18.0) * (m.radius + 18.0);
        self.e_bullets.retain(|eb| eb.pos.distance_squared(m.pos) > clear_r2);
    }

    fn spawn_enemy(&mut self, kind: u8) {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let dist = (screen_width().max(screen_height()) * 0.62 + 80.0).min(ARENA_RADIUS - 80.0);
        let mut pos = self.player_pos + Vec2::from_angle(angle) * dist;
        let limit = ARENA_RADIUS - 45.0;
        let len_sq = pos.length_squared();
        if len_sq > limit * limit {
            pos = (pos / len_sq.sqrt()) * limit;
        }
        self.spawn_enemy_at(kind, pos, true);
    }

    fn spawn_enemy_at(&mut self, kind: u8, pos: Vec2, allow_elite: bool) {
        if self.enemies.len() >= MAX_ENEMIES { return; }
        let diff = self.difficulty();
        let hp_scale = (1.0
            + (self.wave.saturating_sub(1)) as f32 * 0.30
            + if self.wave > 4 { ((self.wave - 4) as f32).powf(1.5) * 0.26 } else { 0.0 })
            * diff.hp_mult;
        let spd_scale = (1.0 + ((self.wave.saturating_sub(1)) as f32 * 0.045).min(0.65)) * diff.spd_mult;
        let elite_chance = if self.wave >= diff.elite_wave {
            (0.10 + (self.wave - diff.elite_wave) as f32 * 0.03 + diff.id as f32 * 0.06).min(0.45)
        } else {
            0.0
        };
        let elite = allow_elite && kind != 3 && rand::gen_range(0.0f32, 1.0) < elite_chance;

        let (mut max_hp, mut speed, mut radius, shoot_cd) = match kind {
            0 => (24.0 * hp_scale, 118.0 * spd_scale, 12.0, 1.5),
            1 => (36.0 * hp_scale, 138.0 * spd_scale, 13.0, 1.5),
            2 => (135.0 * hp_scale, 74.0 * spd_scale, 22.0, 1.5),
            3 => (640.0 * hp_scale, 68.0 * spd_scale, 36.0, 1.4),
            4 => (48.0 * hp_scale, 112.0 * spd_scale, 15.0, rand::gen_range(1.2, 2.0)),
            5 => (42.0 * hp_scale, 158.0 * spd_scale, 14.0, 1.5),
            _ => (105.0 * hp_scale, 92.0 * spd_scale, 19.0, rand::gen_range(1.1, 1.8)),
        };
        if elite {
            max_hp *= 2.2;
            speed *= 1.15;
            radius *= 1.26;
        }
        self.enemies.push(Enemy {
            alive: true,
            kind,
            elite,
            pos,
            hp: max_hp,
            max_hp,
            radius,
            speed,
            flash: 0.0,
            timer: rand::gen_range(0.0, 2.0),
            shoot_cd,
            angle: rand::gen_range(0.0, std::f32::consts::TAU),
            spin: rand::gen_range(-3.5, 3.5),
        });
    }

    fn spawn_particle(&mut self, pos: Vec2, vel: Vec2, life: f32, size: f32, color: Color) {
        if self.particles.len() >= MAX_PARTICLES { return; }
        self.particles.push(Particle { pos, vel, life, max_life: life, size, color });
    }

    fn spawn_ring(&mut self, pos: Vec2, max_radius: f32, color: Color, life: f32) {
        if self.rings.len() >= MAX_RINGS { return; }
        self.rings.push(Ring { pos, radius: 4.0, max_radius, life, max_life: life, color });
    }

    fn spawn_text(&mut self, pos: Vec2, val: String, color: Color) {
        if self.texts.len() >= MAX_TEXTS { return; }
        self.texts.push(DmgText { pos, val, life: 0.55, color });
    }

    fn spawn_gem(&mut self, pos: Vec2, xp: f32, shard: u32) {
        if self.gems.len() >= MAX_GEMS { return; }
        self.gems.push(Gem { pos, xp, shard });
    }

    fn spawn_arc(&mut self, a: Vec2, b: Vec2) {
        if self.arcs.len() >= MAX_ARCS { return; }
        self.arcs.push(ArcBolt { a, b, life: 0.14 });
    }

    fn render_world(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let sx = if self.shake > 0.0 { rand::gen_range(-self.shake, self.shake) } else { 0.0 };
        let sy = if self.shake > 0.0 { rand::gen_range(-self.shake, self.shake) } else { 0.0 };

        // Parallax starfield
        for (pos, z, size) in &self.stars {
            let x = ((pos.x - self.player_pos.x * *z).rem_euclid(3200.0)) - 800.0;
            let y = ((pos.y - self.player_pos.y * *z).rem_euclid(3200.0)) - 800.0;
            if x >= -5.0 && x <= sw + 5.0 && y >= -5.0 && y <= sh + 5.0 {
                draw_rectangle(x, y, *size, *size, Color::from_rgba(140, 210, 255, 110));
            }
        }

        let cam_rect = Rect::new(self.player_pos.x - sw * 0.5 + sx, self.player_pos.y - sh * 0.5 + sy, sw, sh);
        let vmin_x = cam_rect.x - 60.0;
        let vmax_x = cam_rect.x + sw + 60.0;
        let vmin_y = cam_rect.y - 60.0;
        let vmax_y = cam_rect.y + sh + 60.0;

        set_camera(&Camera2D::from_display_rect(cam_rect));

        // Cyber grid
        let grid = 80.0;
        let start_x = ((cam_rect.x / grid).floor() * grid).max(-ARENA_RADIUS);
        let end_x = (cam_rect.x + sw).min(ARENA_RADIUS);
        let start_y = ((cam_rect.y / grid).floor() * grid).max(-ARENA_RADIUS);
        let end_y = (cam_rect.y + sh).min(ARENA_RADIUS);
        let grid_col = Color::from_rgba(5, 217, 232, 18);

        let mut gx = start_x;
        while gx <= end_x {
            draw_line(gx, start_y, gx, end_y, 1.0, grid_col);
            gx += grid;
        }
        let mut gy = start_y;
        while gy <= end_y {
            draw_line(start_x, gy, end_x, gy, 1.0, grid_col);
            gy += grid;
        }

        // Sector landmark rings on enlarged map
        let ring_col = Color::from_rgba(5, 217, 232, 30);
        draw_circle_lines(0.0, 0.0, 950.0, 1.5, ring_col);
        draw_circle_lines(0.0, 0.0, 1900.0, 1.5, ring_col);

        // Arena outer wall
        draw_circle_lines(0.0, 0.0, ARENA_RADIUS, 14.0, Color::from_rgba(255, 42, 109, 55));
        draw_circle_lines(0.0, 0.0, ARENA_RADIUS, 2.5, Color::from_rgba(255, 42, 109, 255));

        // Overload Beacon capture zone
        if self.beacon.active {
            let b = self.beacon;
            if b.pos.x + b.radius >= vmin_x && b.pos.x - b.radius <= vmax_x
                && b.pos.y + b.radius >= vmin_y && b.pos.y - b.radius <= vmax_y
            {
                let gold = Color::from_rgba(249, 200, 14, 255);
                draw_circle(b.pos.x, b.pos.y, b.radius, Color::from_rgba(249, 200, 14, 22));
                draw_circle_lines(b.pos.x, b.pos.y, b.radius, 2.0, Color::from_rgba(249, 200, 14, 150));
                let prog = (b.progress / b.duration).clamp(0.0, 1.0);
                if prog > 0.0 {
                    let segs = 28;
                    let max_a = prog * std::f32::consts::TAU;
                    let start_a = -std::f32::consts::FRAC_PI_2;
                    let r_arc = b.radius - 4.0;
                    for s in 0..segs {
                        let a0 = start_a + (s as f32 / segs as f32) * max_a;
                        let a1 = start_a + ((s + 1) as f32 / segs as f32) * max_a;
                        let p0 = b.pos + Vec2::from_angle(a0) * r_arc;
                        let p1 = b.pos + Vec2::from_angle(a1) * r_arc;
                        draw_line(p0.x, p0.y, p1.x, p1.y, 4.5, gold);
                    }
                }
                draw_poly_lines(b.pos.x, b.pos.y, 4, 16.0, (b.ttl * 2.2).to_degrees(), 2.5, gold);
                draw_centered_text("PRZEKAZNIK", b.pos.x, b.pos.y - b.radius - 8.0, 14.0, gold);
            }
        }

        // Orbital Hazard Strike telegraphs
        for h in &self.hazards {
            if h.pos.x + h.radius < vmin_x || h.pos.x - h.radius > vmax_x
                || h.pos.y + h.radius < vmin_y || h.pos.y - h.radius > vmax_y
            {
                continue;
            }
            let p = 1.0 - (h.timer / h.max_timer).clamp(0.0, 1.0);
            let fill_a = ((0.07 + p * 0.14) * 255.0) as u8;
            draw_circle(h.pos.x, h.pos.y, h.radius, Color::from_rgba(255, 58, 92, fill_a));
            draw_circle_lines(h.pos.x, h.pos.y, h.radius, 2.0, Color::from_rgba(255, 58, 92, 200));
            draw_circle_lines(h.pos.x, h.pos.y, (h.radius * p).max(2.0), 2.5, Color::from_rgba(255, 58, 92, 245));
            let cross = Color::from_rgba(255, 255, 255, 190);
            draw_line(h.pos.x - 10.0, h.pos.y, h.pos.x + 10.0, h.pos.y, 1.5, cross);
            draw_line(h.pos.x, h.pos.y - 10.0, h.pos.x, h.pos.y + 10.0, 1.5, cross);
        }

        // Rings (frustum culled)
        for r in &self.rings {
            if r.pos.x + r.radius < vmin_x || r.pos.x - r.radius > vmax_x || r.pos.y + r.radius < vmin_y || r.pos.y - r.radius > vmax_y {
                continue;
            }
            let mut c = r.color;
            c.a = (r.life / r.max_life).clamp(0.0, 1.0);
            draw_circle_lines(r.pos.x, r.pos.y, r.radius, 3.0, c);
        }

        // Gems (frustum culled)
        for g in &self.gems {
            if g.pos.x < vmin_x || g.pos.x > vmax_x || g.pos.y < vmin_y || g.pos.y > vmax_y {
                continue;
            }
            if g.shard > 0 {
                draw_circle(g.pos.x, g.pos.y, 5.5, Color::from_rgba(249, 200, 14, 255));
            } else {
                draw_circle(g.pos.x, g.pos.y, 3.5, Color::from_rgba(5, 217, 232, 255));
            }
        }

        // Bullets (frustum culled)
        for b in &self.bullets {
            if b.pos.x < vmin_x || b.pos.x > vmax_x || b.pos.y < vmin_y || b.pos.y > vmax_y {
                continue;
            }
            let tail = b.pos - b.vel * 0.028;
            draw_line(b.pos.x, b.pos.y, tail.x, tail.y, 7.0, Color::from_rgba(5, 217, 232, 85));
            draw_line(b.pos.x, b.pos.y, tail.x, tail.y, 2.5, WHITE);
        }

        // Tesla arcs
        for a in &self.arcs {
            let mid = (a.a + a.b) * 0.5 + vec2(rand::gen_range(-12.0, 12.0), rand::gen_range(-12.0, 12.0));
            let col = Color::from_rgba(125, 249, 255, 255);
            draw_line(a.a.x, a.a.y, mid.x, mid.y, 2.5, col);
            draw_line(mid.x, mid.y, a.b.x, a.b.y, 2.5, col);
        }

        // Enemy Bullets (frustum culled)
        for eb in &self.e_bullets {
            if eb.pos.x < vmin_x || eb.pos.x > vmax_x || eb.pos.y < vmin_y || eb.pos.y > vmax_y {
                continue;
            }
            draw_circle(eb.pos.x, eb.pos.y, eb.radius * 1.9, Color::from_rgba(255, 58, 92, 76));
            draw_circle(eb.pos.x, eb.pos.y, eb.radius, Color::from_rgba(255, 58, 92, 255));
            draw_circle(eb.pos.x, eb.pos.y, eb.radius * 0.45, WHITE);
        }

        // Enemies (frustum culled, 7 archetypes + sniper laser + vortex field + elite halo)
        for e in &self.enemies {
            if !e.alive { continue; }
            if e.pos.x + e.radius + 20.0 < vmin_x || e.pos.x - e.radius - 20.0 > vmax_x
                || e.pos.y + e.radius + 20.0 < vmin_y || e.pos.y - e.radius - 20.0 > vmax_y
            {
                continue;
            }

            if e.kind == 4 && e.shoot_cd > 0.0 && e.shoot_cd < 0.45 {
                let la = ((0.35 + (0.45 - e.shoot_cd) * 1.4).clamp(0.0, 1.0) * 255.0) as u8;
                draw_line(e.pos.x, e.pos.y, self.player_pos.x, self.player_pos.y, 1.5, Color::from_rgba(255, 58, 92, la));
            }
            if e.kind == 6 {
                let vr = 44.0 + (e.timer * 5.0).sin() * 6.0;
                draw_circle_lines(e.pos.x, e.pos.y, vr, 1.5, Color::from_rgba(125, 110, 255, 72));
            }

            let enraged_boss = e.kind == 3 && e.hp < e.max_hp * 0.5;
            let base_col = if e.flash > 0.0 {
                WHITE
            } else {
                match e.kind {
                    1 => Color::from_rgba(255, 123, 0, 255),
                    2 => Color::from_rgba(249, 200, 14, 255),
                    3 => if enraged_boss { Color::from_rgba(255, 58, 92, 255) } else { Color::from_rgba(184, 41, 234, 255) },
                    4 => Color::from_rgba(255, 58, 92, 255),
                    5 => Color::from_rgba(170, 255, 45, 255),
                    6 => Color::from_rgba(125, 110, 255, 255),
                    _ => Color::from_rgba(255, 42, 109, 255),
                }
            };
            let sides = match e.kind { 0 => 3, 1 | 5 => 4, 4 => 5, 2 => 6, 6 => 7, _ => 8 };
            if e.elite {
                draw_circle_lines(e.pos.x, e.pos.y, e.radius + 5.0, 1.5, Color::from_rgba(255, 255, 255, 165));
            }
            let mut glow_col = base_col;
            glow_col.a = 0.28;
            let deg = e.angle.to_degrees();
            draw_poly_lines(e.pos.x, e.pos.y, sides, e.radius, deg, 7.0, glow_col);
            draw_poly_lines(e.pos.x, e.pos.y, sides, e.radius, deg, 2.2, base_col);

            if (e.kind == 2 || e.kind == 3 || e.kind == 6 || e.elite) && e.hp < e.max_hp {
                let bw = e.radius * 2.0;
                let ratio = (e.hp / e.max_hp).clamp(0.0, 1.0);
                draw_rectangle(e.pos.x - e.radius, e.pos.y - e.radius - 10.0, bw, 4.0, Color::from_rgba(255, 255, 255, 50));
                draw_rectangle(e.pos.x - e.radius, e.pos.y - e.radius - 10.0, bw * ratio, 4.0, base_col);
            }
        }

        // Orbital saws
        if self.orbit_count > 0 {
            let col = Color::from_rgba(5, 255, 161, 255);
            for i in 0..self.orbit_count {
                let a = self.orbit_angle + (i as f32 * std::f32::consts::TAU) / self.orbit_count as f32;
                let bpos = self.player_pos + Vec2::from_angle(a) * 74.0;
                draw_circle_lines(bpos.x, bpos.y, 8.5, 2.5, col);
            }
        }

        // Phantom Wraith Plasma Mines
        for m in &self.mines {
            if m.pos.x + 28.0 < vmin_x || m.pos.x - 28.0 > vmax_x || m.pos.y + 28.0 < vmin_y || m.pos.y - 28.0 > vmax_y {
                continue;
            }
            let pulse = 18.0 + (m.life * 8.0).sin() * 3.0;
            draw_circle_lines(m.pos.x, m.pos.y, pulse, 1.5, Color::from_rgba(184, 41, 234, 110));
            draw_poly_lines(m.pos.x, m.pos.y, 6, 9.0, (m.life * 2.5).to_degrees(), 2.2, Color::from_rgba(184, 41, 234, 255));
            draw_circle(m.pos.x, m.pos.y, 3.5, Color::from_rgba(255, 42, 109, 255));
        }

        // Player ship (4 distinct hulls + plasma shield aura + Overdrive / Dash glow)
        let p = self.player_pos;
        let ship = self.selected_ship();
        let fwd = Vec2::from_angle(self.aim_angle);
        let right_v = vec2(-fwd.y, fwd.x);

        if self.max_shield > 0.0 && self.shield > 0.5 {
            let sa = ((0.28 + 0.42 * (self.shield / self.max_shield)).clamp(0.0, 1.0) * 255.0) as u8;
            draw_circle_lines(p.x, p.y, 25.0, 2.2, Color::from_rgba(125, 249, 255, sa));
        }

        let ship_col = if self.dash_timer > 0.0 {
            WHITE
        } else if self.overdrive_timer > 0.0 {
            Color::from_rgba(249, 200, 14, 255)
        } else {
            ship.color()
        };
        let glow = if self.overdrive_timer > 0.0 {
            Color::from_rgba(249, 200, 14, 115)
        } else {
            ship.color_a(85)
        };

        let pts: Vec<Vec2> = match ship.id {
            1 => vec![
                p + fwd * 24.0,
                p - fwd * 4.0 + right_v * 16.0,
                p - fwd * 15.0 + right_v * 7.0,
                p - fwd * 8.0,
                p - fwd * 15.0 - right_v * 7.0,
                p - fwd * 4.0 - right_v * 16.0,
            ],
            2 => vec![
                p + fwd * 19.0 + right_v * 6.0,
                p + fwd * 6.0 + right_v * 18.0,
                p - fwd * 15.0 + right_v * 14.0,
                p - fwd * 18.0,
                p - fwd * 15.0 - right_v * 14.0,
                p + fwd * 6.0 - right_v * 18.0,
                p + fwd * 19.0 - right_v * 6.0,
            ],
            3 => vec![
                p + fwd * 22.0,
                p + fwd * 4.0 + right_v * 13.0,
                p - fwd * 17.0 + right_v * 15.0,
                p - fwd * 6.0,
                p - fwd * 17.0 - right_v * 15.0,
                p + fwd * 4.0 - right_v * 13.0,
            ],
            _ => vec![
                p + fwd * 20.0,
                p + Vec2::from_angle(self.aim_angle + 2.4) * 15.0,
                p - fwd * 7.0,
                p + Vec2::from_angle(self.aim_angle - 2.4) * 15.0,
            ],
        };
        let n_pts = pts.len();
        for i in 0..n_pts {
            let a = pts[i];
            let b = pts[(i + 1) % n_pts];
            draw_line(a.x, a.y, b.x, b.y, 7.5, glow);
            draw_line(a.x, a.y, b.x, b.y, 2.5, ship_col);
        }

        // Off-screen Overload Beacon gold diamond indicator
        if self.beacon.active {
            let bdiff = self.beacon.pos - p;
            if bdiff.x.abs() > sw * 0.44 || bdiff.y.abs() > sh * 0.44 {
                let ind = p + bdiff.normalize() * 56.0;
                let deg = bdiff.y.atan2(bdiff.x).to_degrees();
                draw_poly_lines(ind.x, ind.y, 4, 6.5, deg, 2.2, Color::from_rgba(249, 200, 14, 255));
            }
        }

        // Off-screen enemy radar dots
        for (idx, e) in self.enemies.iter().enumerate() {
            if !e.alive { continue; }
            if e.kind != 3 && !e.elite && (idx & 3) != 0 { continue; }
            let diff = e.pos - p;
            if diff.x.abs() > sw * 0.48 || diff.y.abs() > sh * 0.48 {
                let ind = p + diff.normalize() * 46.0;
                let c = if e.kind == 3 {
                    Color::from_rgba(184, 41, 234, 255)
                } else if e.elite {
                    WHITE
                } else {
                    Color::from_rgba(255, 42, 109, 170)
                };
                draw_circle(ind.x, ind.y, if e.kind == 3 { 4.5 } else if e.elite { 3.5 } else { 2.5 }, c);
            }
        }

        // Particles (frustum culled)
        for pt in &self.particles {
            if pt.pos.x < vmin_x || pt.pos.x > vmax_x || pt.pos.y < vmin_y || pt.pos.y > vmax_y {
                continue;
            }
            let mut c = pt.color;
            c.a = (pt.life / pt.max_life).clamp(0.0, 1.0);
            draw_rectangle(pt.pos.x - pt.size * 0.5, pt.pos.y - pt.size * 0.5, pt.size, pt.size, c);
        }

        // Damage numbers (frustum culled)
        for t in &self.texts {
            if t.pos.x < vmin_x || t.pos.x > vmax_x || t.pos.y < vmin_y || t.pos.y > vmax_y {
                continue;
            }
            let mut c = t.color;
            c.a = (t.life * 2.0).clamp(0.0, 1.0);
            draw_text(&t.val, t.pos.x - 10.0, t.pos.y, 18.0, c);
        }

        set_default_camera();
    }

    fn render_hud(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let compact = sw < 520.0;
        let cyan = Color::from_rgba(5, 217, 232, 255);
        let pink = Color::from_rgba(255, 42, 109, 255);
        let gold = Color::from_rgba(249, 200, 14, 255);
        let purple = Color::from_rgba(184, 41, 234, 255);

        let pause_rect = self.pause_rect();

        // Left: HP, optional Plasma SHIELD, & XP bars (responsive width)
        let bar_w = (sw * if compact { 0.25 } else { 0.28 }).clamp(88.0, 168.0);
        let lbl_sz = if compact { 14.0 } else { 17.0 };
        draw_text(&format!("HULL {}/{}", self.hp.ceil() as i32, self.max_hp as i32), 12.0, 22.0, lbl_sz, WHITE);
        draw_rectangle(12.0, 28.0, bar_w, 8.0, Color::from_rgba(255, 255, 255, 25));
        draw_rectangle(12.0, 28.0, bar_w * (self.hp / self.max_hp).clamp(0.0, 1.0), 8.0, pink);
        draw_rectangle_lines(12.0, 28.0, bar_w, 8.0, 1.0, cyan);

        let mut left_y = 40.0;
        if self.max_shield > 0.0 {
            let lcyan = Color::from_rgba(125, 249, 255, 255);
            draw_rectangle(12.0, left_y, bar_w, 6.0, Color::from_rgba(255, 255, 255, 20));
            draw_rectangle(12.0, left_y, bar_w * (self.shield / self.max_shield).clamp(0.0, 1.0), 6.0, lcyan);
            draw_rectangle_lines(12.0, left_y, bar_w, 6.0, 1.0, Color::from_rgba(125, 249, 255, 190));
            left_y += 10.0;
        }

        let lvl_lbl_y = left_y + 12.0;
        let lvl_bar_y = left_y + 18.0;
        draw_text(&format!("LVL {}", self.level), 12.0, lvl_lbl_y, lbl_sz, WHITE);
        draw_rectangle(12.0, lvl_bar_y, bar_w, 8.0, Color::from_rgba(255, 255, 255, 25));
        draw_rectangle(12.0, lvl_bar_y, bar_w * (self.xp / self.next_xp).clamp(0.0, 1.0), 8.0, cyan);
        draw_rectangle_lines(12.0, lvl_bar_y, bar_w, 8.0, 1.0, cyan);

        // Subtle red edge damage flash
        if self.hurt_flash > 0.01 {
            let ha = (self.hurt_flash * 0.55 * 255.0) as u8;
            draw_rectangle_lines(0.0, 0.0, sw, sh, 18.0, Color::from_rgba(255, 58, 92, ha));
        }

        // Right: SHARDS, DIFF/FPS, COMBO & OVERDRIVE right-aligned cleanly before Pause button
        let right_edge = pause_rect.x - 10.0;
        let shard_sz = if compact { 14.0 } else { 17.0 };
        let shard_txt = format!("SHARDS: {}", self.run_shards);
        let shard_w = draw_right_text(&shard_txt, right_edge, 22.0, shard_sz, gold);
        let diff = self.difficulty();
        let meta_right = if diff.id > 0 {
            format!("{} • {} FPS", diff.name, get_fps())
        } else {
            format!("{} FPS", get_fps())
        };
        draw_right_text(
            &meta_right,
            right_edge,
            40.0,
            if compact { 12.0 } else { 14.0 },
            if diff.id > 0 { diff.color() } else { LIGHTGRAY },
        );

        let mut right_row_y = 56.0;
        if self.combo >= 3 {
            let mult = 1.0 + (self.combo as f32 * 0.025).min(1.0);
            draw_right_text(&format!("COMBO {} ({:.2}x)", self.combo, mult), right_edge, right_row_y, if compact { 12.0 } else { 14.0 }, cyan);
            right_row_y += 16.0;
        }
        if self.overdrive_timer > 0.0 {
            draw_right_text(&format!("OVERDRIVE {:.1}s", self.overdrive_timer), right_edge, right_row_y, if compact { 12.0 } else { 14.0 }, gold);
        }

        // Center: WAVE dynamically centered in free gap between Left and Right blocks
        let left_end = 12.0 + bar_w + 10.0;
        let right_start = right_edge - shard_w - 10.0;
        let free_gap = right_start - left_end;
        let wave_txt = format!("WAVE {}", self.wave);
        let timer_txt = format!("{}s{}", (22.0 - self.wave_timer).ceil().max(1.0) as i32, self.wave_event.tag());

        if free_gap >= 92.0 {
            let wave_cx = left_end + free_gap * 0.5;
            draw_centered_text(&wave_txt, wave_cx, 24.0, if compact { 17.0 } else { 21.0 }, pink);
            draw_centered_text(&timer_txt, wave_cx, 42.0, if compact { 12.0 } else { 14.0 }, LIGHTGRAY);
        } else {
            draw_centered_text(&format!("{} • {}", wave_txt, timer_txt), sw * 0.5, lvl_bar_y + 26.0, 15.0, pink);
        }

        // Wave Event / Beacon announcement banner
        if self.banner_timer > 0.0 {
            let alpha = (self.banner_timer / 0.75).clamp(0.0, 1.0);
            let by = if free_gap >= 92.0 { 64.0 } else { lvl_bar_y + 42.0 };
            let bw = (sw - 170.0).min(390.0);
            if bw > 150.0 {
                let bx = sw * 0.5 - bw * 0.5;
                draw_rectangle(bx, by, bw, 40.0, Color::from_rgba(6, 12, 24, (0.76 * alpha * 255.0) as u8));
                let mut bc = self.banner_color;
                bc.a = 0.85 * alpha;
                draw_rectangle_lines(bx, by, bw, 40.0, 1.5, bc);
                bc.a = alpha;
                draw_centered_text(&self.banner_text, sw * 0.5, by + 17.0, if compact { 13.0 } else { 15.0 }, bc);
                draw_centered_text(&self.banner_sub, sw * 0.5, by + 32.0, if compact { 11.0 } else { 12.0 }, Color::from_rgba(226, 232, 240, (alpha * 255.0) as u8));
            }
        }

        // Pause button
        draw_rectangle(pause_rect.x, pause_rect.y, pause_rect.w, pause_rect.h, Color::from_rgba(10, 16, 30, 220));
        draw_rectangle_lines(pause_rect.x, pause_rect.y, pause_rect.w, pause_rect.h, 1.5, cyan);
        draw_centered_text("||", pause_rect.x + pause_rect.w * 0.5, pause_rect.y + pause_rect.h * 0.66, if compact { 16.0 } else { 20.0 }, cyan);

        // Compact tactical minimap
        let map_r = if compact { 26.0 } else { 32.0 };
        let map_cx = 12.0 + map_r;
        let map_cy = lvl_bar_y + 20.0 + map_r;
        draw_circle(map_cx, map_cy, map_r, Color::from_rgba(6, 12, 24, 185));
        draw_circle_lines(map_cx, map_cy, map_r, 1.5, Color::from_rgba(255, 42, 109, 170));
        let map_scale = (map_r - 3.0) / ARENA_RADIUS;
        if self.beacon.active {
            let bmx = map_cx + self.beacon.pos.x * map_scale;
            let bmy = map_cy + self.beacon.pos.y * map_scale;
            draw_rectangle(bmx - 2.5, bmy - 2.5, 5.0, 5.0, gold);
        }
        for e in &self.enemies {
            if !e.alive { continue; }
            let mx = map_cx + e.pos.x * map_scale;
            let my = map_cy + e.pos.y * map_scale;
            let c = if e.kind == 3 { purple } else if e.elite { WHITE } else { Color::from_rgba(255, 42, 109, 190) };
            let dot = if e.kind == 3 { 3.0 } else if e.elite { 2.2 } else { 1.5 };
            draw_rectangle(mx - dot * 0.5, my - dot * 0.5, dot, dot, c);
        }
        draw_circle(
            map_cx + self.player_pos.x * map_scale,
            map_cy + self.player_pos.y * map_scale,
            2.5,
            if self.overdrive_timer > 0.0 { gold } else { cyan },
        );

        // Dash button
        let dc = vec2(sw - 58.0, sh - 62.0);
        let dash_col = if self.dash_cd > 0.0 { Color::from_rgba(70, 90, 120, 180) } else { cyan };
        draw_circle(dc.x, dc.y, 35.0, Color::from_rgba(5, 217, 232, 25));
        draw_circle_lines(dc.x, dc.y, 35.0, 2.0, dash_col);
        draw_centered_text("DASH", dc.x, dc.y - 2.0, 17.0, dash_col);
        let cd_label = if self.dash_cd > 0.0 { format!("{:.1}s", self.dash_cd) } else { "READY".to_string() };
        draw_centered_text(&cd_label, dc.x, dc.y + 15.0, 13.0, dash_col);

        // Joysticks
        if self.left_stick.touch_id.is_none() {
            draw_circle_lines(86.0, sh - 96.0, 42.0, 2.0, Color::from_rgba(5, 217, 232, 65));
            draw_circle(86.0, sh - 96.0, 12.0, Color::from_rgba(5, 217, 232, 45));
        } else {
            draw_circle_lines(self.left_stick.start.x, self.left_stick.start.y, 44.0, 2.0, cyan);
            draw_circle(self.left_stick.cur.x, self.left_stick.cur.y, 16.0, cyan);
        }

        if self.right_stick.touch_id.is_some() {
            draw_circle_lines(self.right_stick.start.x, self.right_stick.start.y, 44.0, 2.0, pink);
            draw_circle(self.right_stick.cur.x, self.right_stick.cur.y, 16.0, pink);
        }
    }
}

fn draw_centered_text(text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, font_size, color);
}

fn draw_right_text(text: &str, rx: f32, y: f32, font_size: f32, color: Color) -> f32 {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, rx - dims.width, y, font_size, color);
    dims.width
}

fn ui_button(rect: Rect, label: &str, sub: Option<&str>, border: Color) -> bool {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::from_rgba(11, 17, 32, 240));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border);
    if let Some(sub_txt) = sub {
        let t_sz = if rect.h < 50.0 { 15.0 } else { 17.0 };
        let s_sz = if rect.h < 50.0 { 12.0 } else { 13.0 };
        draw_centered_text(label, rect.x + rect.w * 0.5, rect.y + rect.h * 0.44, t_sz, border);
        draw_centered_text(sub_txt, rect.x + rect.w * 0.5, rect.y + rect.h * 0.80, s_sz, LIGHTGRAY);
    } else {
        let f_sz = if rect.h < 40.0 { 16.0 } else { 19.0 };
        draw_centered_text(label, rect.x + rect.w * 0.5, rect.y + rect.h * 0.5 + 6.0, f_sz, border);
    }
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        return rect.contains(vec2(mx, my));
    }
    false
}

fn window_conf() -> Conf {
    Conf {
        window_title: "NEON CORE [RUST]".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let sounds = SoundBank::new().await;
    let mut game = Game::new(sounds);

    let cyan = Color::from_rgba(5, 217, 232, 255);
    let pink = Color::from_rgba(255, 42, 109, 255);
    let gold = Color::from_rgba(249, 200, 14, 255);
    let green = Color::from_rgba(5, 255, 161, 255);
    let slate = Color::from_rgba(72, 100, 138, 255);

    loop {
        let dt = get_frame_time().min(0.05);
        clear_background(Color::from_rgba(4, 6, 11, 255));

        let sw = screen_width();
        let sh = screen_height();

        match game.state {
            ScreenState::Menu => {
                let compact = sw < 600.0 || sh < 520.0;
                let purple = Color::from_rgba(184, 41, 234, 255);
                let base_badge = if cfg!(feature = "demo") {
                    "RUST WASM DEMO (MAX WAVE 8)"
                } else {
                    "NATIVE RUST ARM64 • HANGAR & SKLEP"
                };
                let badge = if game.meta.prestige_lvl > 0 {
                    format!("{}  •  PRESTIZ ★{}", base_badge, game.meta.prestige_lvl)
                } else {
                    base_badge.to_string()
                };
                draw_centered_text(&badge, sw * 0.5, if compact { 22.0 } else { 28.0 }, if compact { 12.0 } else { 14.0 }, pink);
                draw_centered_text("NEON CORE", sw * 0.5, if compact { 50.0 } else { 62.0 }, if compact { 30.0 } else { 38.0 }, cyan);
                let sub_hdr = if game.meta.best_wave > 0 {
                    format!("SHARDS: {}  •  REKORD: FALA {} ({} KILLS)", game.meta.shards, game.meta.best_wave, game.meta.best_kills)
                } else {
                    format!("SHARDS: {}", game.meta.shards)
                };
                draw_centered_text(&sub_hdr, sw * 0.5, if compact { 70.0 } else { 84.0 }, if compact { 14.0 } else { 16.0 }, gold);

                // 4 Hangar tabs: BOJOWE | SURVIVAL | ENDGAME | STATKI
                let tab_y = if compact { 80.0 } else { 96.0 };
                let tab_gap = 6.0;
                let total_tab_w = (sw - 24.0).min(580.0);
                let tab_w = (total_tab_w - tab_gap * 3.0) / 4.0;
                let tab_h = if compact { 28.0 } else { 32.0 };
                let tab_start_x = sw * 0.5 - total_tab_w * 0.5;
                let tab_defs: [(u8, &str, Color); 4] = [
                    (0, if compact { "BOJOWE" } else { "BOJOWE (5)" }, cyan),
                    (1, if compact { "SURVIVAL" } else { "SURVIVAL (5)" }, green),
                    (2, if compact { "ENDGAME" } else { "ENDGAME (5)" }, gold),
                    (3, if compact { "STATKI" } else { "STATKI (4)" }, purple),
                ];
                for (t_idx, (tid, lbl, col)) in tab_defs.iter().enumerate() {
                    let rx = tab_start_x + t_idx as f32 * (tab_w + tab_gap);
                    let active = game.shop_tab == *tid;
                    if ui_button(Rect::new(rx, tab_y, tab_w, tab_h), lbl, None, if active { *col } else { slate }) {
                        game.shop_tab = *tid;
                    }
                }

                let cols = if sw >= 580.0 { 2 } else { 1 };
                let start_y = tab_y + tab_h + if compact { 8.0 } else { 11.0 };
                let active_tab = game.shop_tab;
                let cards_end_y: f32;

                if active_tab == 3 {
                    let card_w = if cols == 2 { ((sw - 48.0) * 0.5).min(285.0) } else { (sw - 24.0).min(360.0) };
                    let card_h = if compact { 48.0 } else { 62.0 };
                    let gap_y = if compact { 6.0 } else { 9.0 };
                    let start_x = sw * 0.5 - (cols as f32 * card_w + (cols - 1) as f32 * 10.0) * 0.5;

                    for (i, ship) in SHIPS.iter().enumerate() {
                        let r = i / cols;
                        let c = i % cols;
                        let rx = start_x + c as f32 * (card_w + 10.0);
                        let ry = start_y + r as f32 * (card_h + gap_y);

                        let unlocked = (game.meta.unlocked_ships_mask & (1u32 << ship.id)) != 0;
                        let selected = game.meta.selected_ship == ship.id;
                        let can_buy = !unlocked && game.meta.shards >= ship.cost;
                        let border = if selected {
                            ship.color()
                        } else if unlocked {
                            cyan
                        } else if can_buy {
                            gold
                        } else {
                            slate
                        };
                        let title = if selected {
                            format!("{} [WYBRANY]", ship.name)
                        } else if unlocked {
                            format!("{} [WYBIERZ]", ship.name)
                        } else {
                            format!("{} [{} SHARDS]", ship.name, ship.cost)
                        };

                        if ui_button(Rect::new(rx, ry, card_w, card_h), &title, Some(ship.perk), border) {
                            if unlocked {
                                game.meta.selected_ship = ship.id;
                                game.meta.save();
                                game.sounds.play(&game.sounds.hit, game.settings.sound, 0.55);
                            } else if can_buy {
                                game.meta.shards -= ship.cost;
                                game.meta.unlocked_ships_mask |= 1u32 << ship.id;
                                game.meta.selected_ship = ship.id;
                                game.meta.save();
                                game.sounds.play(&game.sounds.lvlup, game.settings.sound, 0.9);
                            }
                        }
                    }
                    let rows = (SHIPS.len() + cols - 1) / cols;
                    cards_end_y = start_y + rows as f32 * (card_h + gap_y);
                } else {
                    let card_w = if cols == 2 { ((sw - 48.0) * 0.5).min(285.0) } else { (sw - 24.0).min(360.0) };
                    let card_h = if compact { 40.0 } else { 48.0 };
                    let gap_y = if compact { 5.0 } else { 8.0 };
                    let start_x = sw * 0.5 - (cols as f32 * card_w + (cols - 1) as f32 * 10.0) * 0.5;

                    let mut visible_idx = 0usize;
                    for item in SHOP_ITEMS.iter().filter(|it| it.tab == active_tab) {
                        let r = visible_idx / cols;
                        let c = visible_idx % cols;
                        let is_last_odd = cols == 2 && visible_idx == 4;
                        let rx = if is_last_odd { sw * 0.5 - card_w * 0.5 } else { start_x + c as f32 * (card_w + 10.0) };
                        let ry = start_y + r as f32 * (card_h + gap_y);
                        visible_idx += 1;

                        let lvl = game.meta.get_lvl(item.id);
                        let maxed = lvl >= item.max_lvl;
                        let cost = if item.quadratic {
                            (lvl + 1) * (lvl + 1) * item.base_cost
                        } else {
                            (lvl + 1) * item.base_cost
                        };
                        let can_buy = !maxed && game.meta.shards >= cost;
                        let border = if maxed { green } else if can_buy { gold } else { slate };
                        let title = if maxed {
                            format!("{} [MAX]", item.name)
                        } else {
                            format!("{} [{}/{}]", item.name, lvl, item.max_lvl)
                        };
                        let sub = if maxed {
                            format!("{} • MAKSIMUM", item.bonus)
                        } else {
                            format!("{} • {} SHARDS", item.bonus, cost)
                        };

                        if ui_button(Rect::new(rx, ry, card_w, card_h), &title, Some(&sub), border) && can_buy {
                            game.meta.shards -= cost;
                            game.meta.inc_lvl(item.id);
                            game.meta.save();
                            game.sounds.play(&game.sounds.lvlup, game.settings.sound, 0.8);
                        }
                    }
                    let rows = (visible_idx + cols - 1) / cols;
                    cards_end_y = start_y + rows as f32 * (card_h + gap_y);
                }

                // Difficulty selector row
                let diff_y = cards_end_y + if compact { 4.0 } else { 8.0 };
                let diff_gap = 6.0;
                let diff_total_w = (sw - 24.0).min(480.0);
                let diff_w = (diff_total_w - diff_gap * 2.0) / 3.0;
                let diff_h = if compact { 28.0 } else { 34.0 };
                let diff_start_x = sw * 0.5 - diff_total_w * 0.5;
                for (d_idx, df) in DIFFICULTIES.iter().enumerate() {
                    let active = game.meta.difficulty == df.id;
                    let rx = diff_start_x + d_idx as f32 * (diff_w + diff_gap);
                    if ui_button(Rect::new(rx, diff_y, diff_w, diff_h), df.short_tag, None, if active { df.color() } else { slate }) {
                        game.meta.difficulty = df.id;
                        game.meta.save();
                    }
                }

                // Bottom action row
                let btn_y = diff_y + diff_h + if compact { 7.0 } else { 10.0 };
                let btn_w = ((sw - 34.0) * 0.5).min(185.0);
                let btn_h = if compact { 38.0 } else { 44.0 };
                let active_ship = game.selected_ship();
                let start_lbl = format!("START: {}", active_ship.short_name);

                if ui_button(Rect::new(sw * 0.5 - btn_w - 6.0, btn_y, btn_w, btn_h), &start_lbl, None, active_ship.color()) {
                    game.start_run();
                }
                if ui_button(Rect::new(sw * 0.5 + 6.0, btn_y, btn_w, btn_h), "OPCJE", None, gold) {
                    game.prev_state = ScreenState::Menu;
                    game.state = ScreenState::Settings;
                }
            }
            ScreenState::Playing => {
                game.update_playing(dt);
                game.render_world();
                game.render_hud();
            }
            ScreenState::Paused => {
                game.render_world();
                draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(4, 6, 11, 210));
                draw_centered_text("PAUZA", sw * 0.5, sh * 0.28, 38.0, cyan);
                if ui_button(Rect::new(sw * 0.5 - 120.0, sh * 0.38, 240.0, 48.0), "WZNOW", None, cyan) {
                    game.state = ScreenState::Playing;
                }
                if ui_button(Rect::new(sw * 0.5 - 120.0, sh * 0.38 + 60.0, 240.0, 48.0), "OPCJE", None, gold) {
                    game.prev_state = ScreenState::Paused;
                    game.state = ScreenState::Settings;
                }
                if ui_button(Rect::new(sw * 0.5 - 120.0, sh * 0.38 + 120.0, 240.0, 48.0), "PRZERWIJ LOT", None, pink) {
                    game.finish_run(false);
                }
            }
            ScreenState::Settings => {
                draw_centered_text("OPCJE SYSTEMU", sw * 0.5, 65.0, 32.0, cyan);
                let bw = (sw - 40.0).min(320.0);
                let bx = sw * 0.5 - bw * 0.5;

                let s_lbl = if game.settings.sound { "DZWIEK: WLACZONY" } else { "DZWIEK: WYLACZONY" };
                if ui_button(Rect::new(bx, 105.0, bw, 44.0), s_lbl, None, cyan) {
                    game.settings.sound = !game.settings.sound;
                }
                let sh_lbl = if game.settings.shake { "SHAKE: WLACZONY" } else { "SHAKE: WYLACZONY" };
                if ui_button(Rect::new(bx, 158.0, bw, 44.0), sh_lbl, None, cyan) {
                    game.settings.shake = !game.settings.shake;
                }
                let dm_lbl = if game.settings.dmg_nums { "OBRAZENIA: WLACZONE" } else { "OBRAZENIA: WYLACZONE" };
                if ui_button(Rect::new(bx, 211.0, bw, 44.0), dm_lbl, None, cyan) {
                    game.settings.dmg_nums = !game.settings.dmg_nums;
                }
                let st_lbl = if game.settings.fixed_stick { "DRAZEK: STALY" } else { "DRAZEK: PLYWAJACY" };
                if ui_button(Rect::new(bx, 264.0, bw, 44.0), st_lbl, None, cyan) {
                    game.settings.fixed_stick = !game.settings.fixed_stick;
                }
                let ai_lbl = if game.settings.auto_aim { "AUTO-AIM: WLACZONY" } else { "AUTO-AIM: WYLACZONY" };
                if ui_button(Rect::new(bx, 317.0, bw, 44.0), ai_lbl, None, cyan) {
                    game.settings.auto_aim = !game.settings.auto_aim;
                }
                if ui_button(Rect::new(bx, 378.0, bw, 48.0), "WROC", None, gold) {
                    game.state = game.prev_state;
                }
            }
            ScreenState::LevelUp => {
                game.render_world();
                draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(4, 6, 11, 215));
                draw_centered_text("OVERCLOCK RDZENIA", sw * 0.5, 65.0, 30.0, cyan);
                let cw = (sw - 40.0).min(340.0);
                let choices = game.level_choices;
                for (i, opt) in choices.iter().enumerate() {
                    let rect = Rect::new(sw * 0.5 - cw * 0.5, 105.0 + i as f32 * 74.0, cw, 62.0);
                    if ui_button(rect, opt.title, Some(opt.desc), cyan) {
                        game.apply_upgrade(opt.id);
                        break;
                    }
                }
            }
            ScreenState::GameOver { victory } => {
                let title = if victory { "SEKTOR DEMO OCZYSZCZONY" } else { "RDZEN ZNISZCZONY" };
                let diff = game.difficulty();
                let ship = game.selected_ship();
                draw_centered_text(title, sw * 0.5, sh * 0.26, 32.0, if victory { cyan } else { pink });
                draw_centered_text(
                    &format!("{}  •  {} ({:.1}x)", ship.name, diff.name, diff.shard_mult),
                    sw * 0.5,
                    sh * 0.26 + 32.0,
                    16.0,
                    diff.color(),
                );
                if game.is_new_record {
                    draw_centered_text("★ NOWY REKORD OSOBISTY! ★", sw * 0.5, sh * 0.26 + 58.0, 18.0, gold);
                }
                let stat_y = sh * 0.26 + if game.is_new_record { 90.0 } else { 70.0 };
                draw_centered_text(
                    &format!("FALA: {}  |  POZIOM: {}", game.wave, game.level),
                    sw * 0.5,
                    stat_y,
                    20.0,
                    WHITE,
                );
                draw_centered_text(
                    &format!("ZNISZCZENI: {}  |  SHARDS: +{}", game.kills, game.run_shards),
                    sw * 0.5,
                    stat_y + 30.0,
                    20.0,
                    gold,
                );
                draw_centered_text(
                    &format!("NAJLEPSZA FALA: {}  |  REKORD KILLS: {}", game.meta.best_wave, game.meta.best_kills),
                    sw * 0.5,
                    stat_y + 56.0,
                    15.0,
                    LIGHTGRAY,
                );
                if ui_button(Rect::new(sw * 0.5 - 135.0, stat_y + 80.0, 270.0, 50.0), "POWROT DO HANGARU", None, cyan) {
                    game.state = ScreenState::Menu;
                }
            }
        }

        next_frame().await;
    }
}
