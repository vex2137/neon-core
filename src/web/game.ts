import { SoundBank } from './audio';

const ARENA_RADIUS = 2800;
const TAU = Math.PI * 2;
const SAVE_KEY = 'neon_core_save_v1';

// Spatial hash grid constants for O(1) enemy queries
const CELL_SIZE = 128;
const GRID_DIM = 48; // covers [-3072, +3072] comfortably around ARENA_RADIUS=2800
const GRID_HALF = GRID_DIM >> 1;
const GRID_CELLS = GRID_DIM * GRID_DIM;

const MAX_BULLETS = 520;
const MAX_EBULLETS = 320;
const MAX_HAZARDS = 28;
const MAX_MINES = 18;
const MAX_ENEMIES = 420;
const MAX_PARTICLES = 640;
const MAX_RINGS = 48;
const MAX_TEXTS = 80;
const MAX_GEMS = 380;
const MAX_ARCS = 48;

interface Vec2 {
  x: number;
  y: number;
}

interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

type ScreenState =
  | { kind: 'Menu' }
  | { kind: 'Playing' }
  | { kind: 'Paused' }
  | { kind: 'Settings' }
  | { kind: 'LevelUp' }
  | { kind: 'GameOver'; victory: boolean };

interface Settings {
  sound: boolean;
  shake: boolean;
  dmgNums: boolean;
  fixedStick: boolean;
  autoAim: boolean;
}

interface MetaSave {
  shards: number;
  dmgLvl: number;
  rateLvl: number;
  hpLvl: number;
  spdLvl: number;
  critLvl: number;
  regenLvl: number;
  magLvl: number;
  dashLvl: number;
  greedLvl: number;
  droneLvl: number;
  // Endgame upgrades & progression (indices 11..20 in save CSV)
  shieldLvl: number;
  startSalvoLvl: number;
  supernovaLvl: number;
  overdriveLvl: number;
  prestigeLvl: number;
  selectedShip: number;
  unlockedShipsMask: number;
  difficulty: number;
  bestWave: number;
  bestKills: number;
}

type ShopUpgradeKey =
  | 'dmgLvl'
  | 'rateLvl'
  | 'hpLvl'
  | 'spdLvl'
  | 'critLvl'
  | 'regenLvl'
  | 'magLvl'
  | 'dashLvl'
  | 'greedLvl'
  | 'droneLvl'
  | 'shieldLvl'
  | 'startSalvoLvl'
  | 'supernovaLvl'
  | 'overdriveLvl'
  | 'prestigeLvl';

interface ShopItem {
  key: ShopUpgradeKey;
  tab: 0 | 1 | 2; // 0: BOJOWE, 1: SURVIVAL, 2: ENDGAME
  name: string;
  bonus: string;
  baseCost: number;
  maxLvl: number;
  quadratic?: boolean;
}

interface ShipSpec {
  id: number;
  name: string;
  role: string;
  perk: string;
  cost: number;
  rgb: [number, number, number];
}

interface DifficultySpec {
  id: number;
  name: string;
  shortTag: string;
  shardMult: number;
  hpMult: number;
  spdMult: number;
  dmgMult: number;
  eliteWave: number;
  doubleBossWave: number;
  rgb: [number, number, number];
}

const SHOP_ITEMS: ShopItem[] = [
  // BOJOWE
  { key: 'dmgLvl', tab: 0, name: 'MOC RDZENIA', bonus: '+15% DMG', baseCost: 15, maxLvl: 15 },
  { key: 'rateLvl', tab: 0, name: 'SZYBKOSTRZELNOSC', bonus: '+9% ROF', baseCost: 15, maxLvl: 12 },
  { key: 'critLvl', tab: 0, name: 'KRYTYCZNY RDZEN', bonus: '+8% CRIT 2x', baseCost: 20, maxLvl: 8 },
  { key: 'droneLvl', tab: 0, name: 'DRON STARTOWY', bonus: '+1 Ostrze start', baseCost: 40, maxLvl: 4 },
  { key: 'dashLvl', tab: 0, name: 'DOPALACZ DASH', bonus: '-12% CD uniku', baseCost: 20, maxLvl: 6 },
  // SURVIVAL
  { key: 'hpLvl', tab: 1, name: 'PANCERZ KADLUBA', bonus: '+30 Max HP', baseCost: 15, maxLvl: 15 },
  { key: 'regenLvl', tab: 1, name: 'REGENERACJA NANO', bonus: '+1.5 HP/s', baseCost: 20, maxLvl: 10 },
  { key: 'spdLvl', tab: 1, name: 'SILNIKI MANEWROWE', bonus: '+8% Predkosc', baseCost: 15, maxLvl: 10 },
  { key: 'magLvl', tab: 1, name: 'MAGNES ZBIERANIA', bonus: '+35% Zasieg', baseCost: 15, maxLvl: 8 },
  { key: 'greedLvl', tab: 1, name: 'EKSTRAKTOR SHARDOW', bonus: '+20% Shards/XP', baseCost: 25, maxLvl: 8 },
  // ENDGAME (high-cost shard sinks with quadratic cost scaling)
  { key: 'shieldLvl', tab: 2, name: 'TARCZA PLAZMOWA', bonus: '+35 Max Tarczy + Regen', baseCost: 400, maxLvl: 10, quadratic: true },
  { key: 'startSalvoLvl', tab: 2, name: 'OVERCLOCK SALWY', bonus: '+1 Salwa start (+Przebicie)', baseCost: 600, maxLvl: 5, quadratic: true },
  { key: 'supernovaLvl', tab: 2, name: 'PODWOJNA SUPERNOVA', bonus: 'EMP niszczy pociski + Start', baseCost: 450, maxLvl: 8, quadratic: true },
  { key: 'overdriveLvl', tab: 2, name: 'REAKTOR OVERDRIVE', bonus: '+1.2s, +12% DMG, Boss OD', baseCost: 350, maxLvl: 8, quadratic: true },
  { key: 'prestigeLvl', tab: 2, name: 'RDZEN PRESTIZU', bonus: '+10% DMG/+8% HP/+10% Shards', baseCost: 500, maxLvl: 25, quadratic: true },
];

const SHIPS: ShipSpec[] = [
  {
    id: 0,
    name: 'INTERCEPTOR',
    role: 'KLASYCZNY MYSLIWIEC',
    perk: 'Zbalansowane statystyki bojowe',
    cost: 0,
    rgb: [5, 217, 232],
  },
  {
    id: 1,
    name: 'VIPER MK-II',
    role: 'SZYBKI SZTURMOWIEC',
    perk: '+22% Spd, +28% ROF, +1 Salwa, -15% HP',
    cost: 1500,
    rgb: [5, 255, 161],
  },
  {
    id: 2,
    name: 'TITAN DREADNOUGHT',
    role: 'CIEZKI PANCERNIK',
    perk: '+70% HP, +60 Tarcza, Strzelba 4x Plazma',
    cost: 4500,
    rgb: [249, 200, 14],
  },
  {
    id: 3,
    name: 'PHANTOM WRAITH',
    role: 'DUCH TAKTYCZNY',
    perk: '-32% Dash CD, +15% CRIT, Miny po Dashu',
    cost: 9000,
    rgb: [184, 41, 234],
  },
];

const DIFFICULTIES: DifficultySpec[] = [
  {
    id: 0,
    name: 'NORMAL',
    shortTag: 'NORMAL 1x',
    shardMult: 1.0,
    hpMult: 1.0,
    spdMult: 1.0,
    dmgMult: 1.0,
    eliteWave: 4,
    doubleBossWave: 9,
    rgb: [5, 217, 232],
  },
  {
    id: 1,
    name: 'HARDCORE',
    shortTag: 'HARDCORE 1.6x',
    shardMult: 1.6,
    hpMult: 1.45,
    spdMult: 1.14,
    dmgMult: 1.35,
    eliteWave: 2,
    doubleBossWave: 6,
    rgb: [255, 123, 0],
  },
  {
    id: 2,
    name: 'NIGHTMARE',
    shortTag: 'NIGHTMARE 2.5x',
    shardMult: 2.5,
    hpMult: 2.2,
    spdMult: 1.26,
    dmgMult: 1.75,
    eliteWave: 1,
    doubleBossWave: 3,
    rgb: [255, 58, 92],
  },
];

interface Bullet {
  x: number;
  y: number;
  vx: number;
  vy: number;
  dmg: number;
  life: number;
  pierce: number;
}

interface EnemyBullet {
  x: number;
  y: number;
  vx: number;
  vy: number;
  dmg: number;
  life: number;
  radius: number;
}

interface Mine {
  x: number;
  y: number;
  life: number;
  dmg: number;
  radius: number;
}

interface HazardZone {
  x: number;
  y: number;
  radius: number;
  timer: number;
  maxTimer: number;
  dmg: number;
}

interface BeaconObjective {
  active: boolean;
  x: number;
  y: number;
  radius: number;
  progress: number;
  duration: number;
  ttl: number;
}

type WaveEventKind = 'NORMAL' | 'SWARM' | 'ORBITAL' | 'SNIPER' | 'BOMBER' | 'BOSS';

interface Enemy {
  alive: boolean;
  kind: number; // 0: Swarmer, 1: Dasher, 2: Tank, 3: Boss, 4: Sniper, 5: Bomber, 6: Vortex
  elite: boolean;
  x: number;
  y: number;
  hp: number;
  maxHp: number;
  radius: number;
  speed: number;
  flash: number;
  timer: number;
  shootCd: number;
  angle: number;
  spin: number;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  size: number;
  rgb: [number, number, number];
}

interface Ring {
  x: number;
  y: number;
  radius: number;
  maxRadius: number;
  life: number;
  maxLife: number;
  rgb: [number, number, number];
}

interface DmgText {
  x: number;
  y: number;
  val: string;
  life: number;
  rgb: [number, number, number];
}

interface Gem {
  x: number;
  y: number;
  xp: number;
  shard: number;
}

interface ArcBolt {
  ax: number;
  ay: number;
  bx: number;
  by: number;
  life: number;
}

interface UpgradeCard {
  id: number;
  title: string;
  desc: string;
}

interface Stick {
  touchId: number | null;
  startX: number;
  startY: number;
  curX: number;
  curY: number;
  dirX: number;
  dirY: number;
}

const ALL_UPGRADES: UpgradeCard[] = [
  { id: 0, title: 'ROZDZIELACZ LUF', desc: '+1 Pocisk na salwe' },
  { id: 1, title: 'PLAZMA OVERCHARGE', desc: '+25% Obrazen broni' },
  { id: 2, title: 'CEWKI CYKLERA', desc: '+20% Szybkostrzelnosci' },
  { id: 3, title: 'PENETRATOR', desc: '+1 Przebicie wrogow' },
  { id: 4, title: 'PILA ORBITALNA', desc: '+1 Wirujace ostrze' },
  { id: 5, title: 'LUK TESLI', desc: 'Blyskawice lancuchowe' },
  { id: 6, title: 'IMPULS EMP', desc: 'Radialna fala uderzeniowa' },
  { id: 7, title: 'CHLODZENIE DASH', desc: '-25% Cooldownu uniku' },
  { id: 8, title: 'NANONAPRAWA', desc: '+40% HP i +25 Max HP' },
  { id: 9, title: 'MAGNES RDZENIA', desc: '+45% Zasieg zbierania' },
  { id: 10, title: 'TURBO KRYTYK', desc: '+15% Szansy na CRIT 2x' },
  { id: 11, title: 'AUTONAPRAWA', desc: '+3 HP/s regeneracji' },
];

const RGB_CYAN: [number, number, number] = [5, 217, 232];
const RGB_PINK: [number, number, number] = [255, 42, 109];
const RGB_GOLD: [number, number, number] = [249, 200, 14];
const RGB_ORANGE: [number, number, number] = [255, 123, 0];
const RGB_PURPLE: [number, number, number] = [184, 41, 234];
const RGB_LIGHT_CYAN: [number, number, number] = [125, 249, 255];
const RGB_GREEN: [number, number, number] = [5, 255, 161];
const RGB_LIME: [number, number, number] = [170, 255, 45];
const RGB_CRIMSON: [number, number, number] = [255, 58, 92];
const RGB_INDIGO: [number, number, number] = [125, 110, 255];
const RGB_WHITE: [number, number, number] = [255, 255, 255];
const RGB_SLATE: [number, number, number] = [72, 100, 138];

function rgba(rgb: [number, number, number], alpha = 1): string {
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

function randRange(min: number, max: number): number {
  return min + Math.random() * (max - min);
}

function remEuclid(a: number, b: number): number {
  return ((a % b) + b) % b;
}

function rectContains(r: Rect, x: number, y: number): boolean {
  return x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h;
}

function makeStick(): Stick {
  return { touchId: null, startX: 0, startY: 0, curX: 0, curY: 0, dirX: 0, dirY: 0 };
}

function defaultMeta(): MetaSave {
  return {
    shards: 0,
    dmgLvl: 0,
    rateLvl: 0,
    hpLvl: 0,
    spdLvl: 0,
    critLvl: 0,
    regenLvl: 0,
    magLvl: 0,
    dashLvl: 0,
    greedLvl: 0,
    droneLvl: 0,
    shieldLvl: 0,
    startSalvoLvl: 0,
    supernovaLvl: 0,
    overdriveLvl: 0,
    prestigeLvl: 0,
    selectedShip: 0,
    unlockedShipsMask: 1,
    difficulty: 0,
    bestWave: 0,
    bestKills: 0,
  };
}

function loadMeta(): MetaSave {
  const m = defaultMeta();
  try {
    const raw = window.localStorage.getItem(SAVE_KEY);
    if (raw) {
      const parts = raw.split(',').map((p) => Number.parseInt(p.trim(), 10));
      if (parts.length >= 5 && parts.every((n) => Number.isFinite(n) && n >= 0)) {
        m.shards = parts[0];
        m.dmgLvl = parts[1];
        m.rateLvl = parts[2];
        m.hpLvl = parts[3];
        m.spdLvl = parts[4];
        if (parts.length >= 11) {
          m.critLvl = parts[5];
          m.regenLvl = parts[6];
          m.magLvl = parts[7];
          m.dashLvl = parts[8];
          m.greedLvl = parts[9];
          m.droneLvl = parts[10];
        }
        if (parts.length >= 21) {
          m.shieldLvl = parts[11];
          m.startSalvoLvl = parts[12];
          m.supernovaLvl = parts[13];
          m.overdriveLvl = parts[14];
          m.prestigeLvl = parts[15];
          m.unlockedShipsMask = Math.max(1, parts[17] | 1);
          m.selectedShip =
            parts[16] >= 0 && parts[16] < SHIPS.length && (m.unlockedShipsMask & (1 << parts[16])) !== 0
              ? parts[16]
              : 0;
          m.difficulty = Math.min(DIFFICULTIES.length - 1, Math.max(0, parts[18]));
          m.bestWave = parts[19];
          m.bestKills = parts[20];
        }
      }
    }
  } catch {
    // Ignore storage errors
  }
  return m;
}

function saveMeta(m: MetaSave): void {
  try {
    window.localStorage.setItem(
      SAVE_KEY,
      [
        m.shards,
        m.dmgLvl,
        m.rateLvl,
        m.hpLvl,
        m.spdLvl,
        m.critLvl,
        m.regenLvl,
        m.magLvl,
        m.dashLvl,
        m.greedLvl,
        m.droneLvl,
        m.shieldLvl,
        m.startSalvoLvl,
        m.supernovaLvl,
        m.overdriveLvl,
        m.prestigeLvl,
        m.selectedShip,
        m.unlockedShipsMask,
        m.difficulty,
        m.bestWave,
        m.bestKills,
      ].join(',')
    );
  } catch {
    // Ignore storage errors
  }
}

export class NeonCoreGame {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private width = 960;
  private height = 600;
  private dpr = 1;
  private cachedFontSize = 0;

  private state: ScreenState = { kind: 'Menu' };
  private prevState: ScreenState = { kind: 'Menu' };
  private shopTab: 0 | 1 | 2 | 3 = 0;
  private settings: Settings = {
    sound: true,
    shake: true,
    dmgNums: true,
    fixedStick: false,
    autoAim: true,
  };
  private meta: MetaSave = loadMeta();
  private sounds = new SoundBank();

  // Dense O(1) swap-remove pools (zero runtime allocations)
  private bullets: Bullet[];
  private bulletActive = 0;

  private eBullets: EnemyBullet[];
  private eBulletActive = 0;

  private hazards: HazardZone[];
  private hazardActive = 0;

  private mines: Mine[];
  private mineActive = 0;

  private enemies: Enemy[];
  private enemyActive = 0;
  private cellHead = new Int32Array(GRID_CELLS);
  private enemyNext = new Int32Array(MAX_ENEMIES);

  private particles: Particle[];
  private particleActive = 0;

  private rings: Ring[];
  private ringActive = 0;

  private texts: DmgText[];
  private textActive = 0;

  private gems: Gem[];
  private gemActive = 0;

  private arcs: ArcBolt[];
  private arcActive = 0;

  private stars: Array<{ x: number; y: number; z: number; size: number }>;

  private playerX = 0;
  private playerY = 0;
  private playerVx = 0;
  private playerVy = 0;
  private aimAngle = 0;
  private thrustAccum = 0;

  private dashTimer = 0;
  private dashDuration = 0.18;
  private dashCd = 0;
  private dashMaxCd = 1.6;
  private dashDirX = 1;
  private dashDirY = 0;

  private hp = 100;
  private maxHp = 100;
  private shield = 0;
  private maxShield = 0;
  private shieldDelay = 0;
  private hpRegen = 0;
  private critChance = 0;
  private greedMult = 1;
  private xp = 0;
  private nextXp = 25;
  private level = 1;
  private wave = 1;
  private waveTimer = 0;
  private waveEvent: WaveEventKind = 'NORMAL';
  private bannerText = '';
  private bannerSub = '';
  private bannerTimer = 0;
  private bannerRgb: [number, number, number] = RGB_CYAN;
  private hazardSpawnTimer = 4.5;
  private beaconSpawnTimer = 14;
  private beacon: BeaconObjective = {
    active: false,
    x: 0,
    y: 0,
    radius: 82,
    progress: 0,
    duration: 2.4,
    ttl: 0,
  };
  private overdriveTimer = 0;
  private combo = 0;
  private comboTimer = 0;
  private hurtFlash = 0;
  private spawnAccum = 0;
  private runShards = 0;
  private kills = 0;
  private isNewRecord = false;
  private shake = 0;

  private moveSpeed = 275;
  private fireCd = 0;
  private fireInterval = 0.21;
  private bulletDmg = 18;
  private bulletCount = 1;
  private bulletPierce = 1;
  private bulletSpread = 0.11;
  private magnetRadius = 130;

  private orbitCount = 0;
  private orbitAngle = 0;
  private orbitDmg = 16;
  private teslaLvl = 0;
  private teslaTimer = 0;
  private novaLvl = 0;
  private novaTimer = 0;

  private leftStick: Stick = makeStick();
  private rightStick: Stick = makeStick();
  private levelChoices: UpgradeCard[] = [ALL_UPGRADES[0], ALL_UPGRADES[1], ALL_UPGRADES[2]];

  private keysDown = new Set<string>();
  private keysPressed = new Set<string>();
  private clickPos: Vec2 | null = null;
  private pointerPos: Vec2 = { x: 0, y: 0 };
  private pointerDown = false;

  private fps = 60;
  private fpsAccum = 0;
  private fpsFrames = 0;
  private lastTime = 0;
  private rafId = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) {
      throw new Error('Could not initialize 2D context');
    }
    this.ctx = ctx;

    this.bullets = Array.from({ length: MAX_BULLETS }, () => ({
      x: 0,
      y: 0,
      vx: 0,
      vy: 0,
      dmg: 0,
      life: 0,
      pierce: 0,
    }));
    this.eBullets = Array.from({ length: MAX_EBULLETS }, () => ({
      x: 0,
      y: 0,
      vx: 0,
      vy: 0,
      dmg: 0,
      life: 0,
      radius: 5,
    }));
    this.hazards = Array.from({ length: MAX_HAZARDS }, () => ({
      x: 0,
      y: 0,
      radius: 64,
      timer: 0,
      maxTimer: 1.2,
      dmg: 24,
    }));
    this.mines = Array.from({ length: MAX_MINES }, () => ({
      x: 0,
      y: 0,
      life: 0,
      dmg: 0,
      radius: 135,
    }));
    this.enemies = Array.from({ length: MAX_ENEMIES }, () => ({
      alive: false,
      kind: 0,
      elite: false,
      x: 0,
      y: 0,
      hp: 0,
      maxHp: 0,
      radius: 12,
      speed: 100,
      flash: 0,
      timer: 0,
      shootCd: 1.5,
      angle: 0,
      spin: 0,
    }));
    this.particles = Array.from({ length: MAX_PARTICLES }, () => ({
      x: 0,
      y: 0,
      vx: 0,
      vy: 0,
      life: 0,
      maxLife: 1,
      size: 3,
      rgb: RGB_CYAN,
    }));
    this.rings = Array.from({ length: MAX_RINGS }, () => ({
      x: 0,
      y: 0,
      radius: 0,
      maxRadius: 100,
      life: 0,
      maxLife: 0.3,
      rgb: RGB_CYAN,
    }));
    this.texts = Array.from({ length: MAX_TEXTS }, () => ({
      x: 0,
      y: 0,
      val: '',
      life: 0,
      rgb: RGB_WHITE,
    }));
    this.gems = Array.from({ length: MAX_GEMS }, () => ({
      x: 0,
      y: 0,
      xp: 0,
      shard: 0,
    }));
    this.arcs = Array.from({ length: MAX_ARCS }, () => ({
      ax: 0,
      ay: 0,
      bx: 0,
      by: 0,
      life: 0,
    }));
    this.stars = Array.from({ length: 160 }, () => ({
      x: randRange(-1600, 1600),
      y: randRange(-1600, 1600),
      z: Math.random() < 0.6 ? 0.25 : 0.55,
      size: randRange(1.5, 2.6),
    }));

    this.resize();
    this.bindEvents();
  }

  public start(): void {
    this.lastTime = performance.now();
    const loop = (now: number) => {
      const rawDt = (now - this.lastTime) / 1000;
      this.lastTime = now;
      const dt = Math.min(Math.max(rawDt, 0.0001), 0.05);

      this.fpsAccum += rawDt;
      this.fpsFrames += 1;
      if (this.fpsAccum >= 0.4) {
        this.fps = Math.round(this.fpsFrames / this.fpsAccum);
        this.fpsAccum = 0;
        this.fpsFrames = 0;
      }

      this.tick(dt);
      this.keysPressed.clear();
      this.clickPos = null;
      this.rafId = requestAnimationFrame(loop);
    };
    this.rafId = requestAnimationFrame(loop);
  }

  public destroy(): void {
    cancelAnimationFrame(this.rafId);
  }

  private resize = (): void => {
    this.dpr = Math.min(window.devicePixelRatio || 1, 2);
    this.width = window.innerWidth;
    this.height = window.innerHeight;
    this.canvas.width = Math.floor(this.width * this.dpr);
    this.canvas.height = Math.floor(this.height * this.dpr);
    this.cachedFontSize = 0;
  };

  private getPauseRect(): Rect {
    const size = this.width < 460 ? 36 : 42;
    return { x: this.width - size - 10, y: 10, w: size, h: size };
  }

  private bindEvents(): void {
    window.addEventListener('resize', this.resize);

    window.addEventListener('keydown', (e) => {
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space'].includes(e.code)) {
        e.preventDefault();
      }
      if (!this.keysDown.has(e.code)) {
        this.keysPressed.add(e.code);
      }
      this.keysDown.add(e.code);
      this.sounds.unlock();
    });

    window.addEventListener('keyup', (e) => {
      this.keysDown.delete(e.code);
    });

    this.canvas.addEventListener('mousedown', (e) => {
      if (e.button !== 0) return;
      this.sounds.unlock();
      const pt = this.clientToCanvas(e.clientX, e.clientY);
      this.pointerPos = pt;
      this.pointerDown = true;
      this.clickPos = pt;
      if (this.state.kind === 'Playing') {
        this.handlePlayPress(pt.x, pt.y);
      }
    });

    window.addEventListener('mousemove', (e) => {
      const pt = this.clientToCanvas(e.clientX, e.clientY);
      this.pointerPos = pt;
    });

    window.addEventListener('mouseup', () => {
      this.pointerDown = false;
    });

    this.canvas.addEventListener(
      'touchstart',
      (e) => {
        e.preventDefault();
        this.sounds.unlock();
        for (let i = 0; i < e.changedTouches.length; i++) {
          const t = e.changedTouches[i];
          const pt = this.clientToCanvas(t.clientX, t.clientY);
          if (this.state.kind !== 'Playing') {
            this.clickPos = pt;
            continue;
          }
          this.handleTouchStart(t.identifier, pt.x, pt.y);
        }
      },
      { passive: false }
    );

    this.canvas.addEventListener(
      'touchmove',
      (e) => {
        e.preventDefault();
        for (let i = 0; i < e.changedTouches.length; i++) {
          const t = e.changedTouches[i];
          const pt = this.clientToCanvas(t.clientX, t.clientY);
          this.handleTouchMove(t.identifier, pt.x, pt.y);
        }
      },
      { passive: false }
    );

    const endTouch = (e: TouchEvent) => {
      e.preventDefault();
      for (let i = 0; i < e.changedTouches.length; i++) {
        const id = e.changedTouches[i].identifier;
        if (this.leftStick.touchId === id) {
          this.leftStick.touchId = null;
          this.leftStick.dirX = 0;
          this.leftStick.dirY = 0;
        } else if (this.rightStick.touchId === id) {
          this.rightStick.touchId = null;
          this.rightStick.dirX = 0;
          this.rightStick.dirY = 0;
        }
      }
    };

    this.canvas.addEventListener('touchend', endTouch, { passive: false });
    this.canvas.addEventListener('touchcancel', endTouch, { passive: false });
  }

  private clientToCanvas(cx: number, cy: number): Vec2 {
    const rect = this.canvas.getBoundingClientRect();
    return { x: cx - rect.left, y: cy - rect.top };
  }

  private handlePlayPress(x: number, y: number): void {
    const pauseRect = this.getPauseRect();
    if (rectContains(pauseRect, x, y)) {
      this.state = { kind: 'Paused' };
      return;
    }
    const dx = x - (this.width - 58);
    const dy = y - (this.height - 62);
    if (dx * dx + dy * dy <= 44 * 44) {
      this.triggerDash();
    }
  }

  private handleTouchStart(id: number, x: number, y: number): void {
    const pauseRect = this.getPauseRect();
    if (rectContains(pauseRect, x, y)) {
      this.state = { kind: 'Paused' };
      return;
    }
    const ddx = x - (this.width - 58);
    const ddy = y - (this.height - 62);
    if (ddx * ddx + ddy * ddy <= 44 * 44) {
      this.triggerDash();
      return;
    }
    const wantMove = this.settings.autoAim
      ? this.leftStick.touchId === null
      : x < this.width * 0.5 && this.leftStick.touchId === null;

    if (wantMove) {
      const startX = this.settings.fixedStick ? 86 : x;
      const startY = this.settings.fixedStick ? this.height - 96 : y;
      this.leftStick.touchId = id;
      this.leftStick.startX = startX;
      this.leftStick.startY = startY;
      this.leftStick.curX = x;
      this.leftStick.curY = y;
      const dx = x - startX;
      const dy = y - startY;
      const lenSq = dx * dx + dy * dy;
      if (lenSq > 4) {
        const len = Math.sqrt(lenSq);
        const s = Math.min(len, 44) / 44;
        this.leftStick.dirX = (dx / len) * s;
        this.leftStick.dirY = (dy / len) * s;
      } else {
        this.leftStick.dirX = 0;
        this.leftStick.dirY = 0;
      }
    } else if (this.rightStick.touchId === null) {
      this.rightStick.touchId = id;
      this.rightStick.startX = x;
      this.rightStick.startY = y;
      this.rightStick.curX = x;
      this.rightStick.curY = y;
      this.rightStick.dirX = 0;
      this.rightStick.dirY = 0;
    }
  }

  private handleTouchMove(id: number, x: number, y: number): void {
    if (this.leftStick.touchId === id) {
      this.leftStick.curX = x;
      this.leftStick.curY = y;
      const dx = x - this.leftStick.startX;
      const dy = y - this.leftStick.startY;
      const lenSq = dx * dx + dy * dy;
      if (lenSq > 4) {
        const len = Math.sqrt(lenSq);
        const s = Math.min(len, 44) / 44;
        this.leftStick.dirX = (dx / len) * s;
        this.leftStick.dirY = (dy / len) * s;
      } else {
        this.leftStick.dirX = 0;
        this.leftStick.dirY = 0;
      }
    } else if (this.rightStick.touchId === id) {
      this.rightStick.curX = x;
      this.rightStick.curY = y;
      const dx = x - this.rightStick.startX;
      const dy = y - this.rightStick.startY;
      const lenSq = dx * dx + dy * dy;
      if (lenSq > 16) {
        const len = Math.sqrt(lenSq);
        this.rightStick.dirX = dx / len;
        this.rightStick.dirY = dy / len;
      } else {
        this.rightStick.dirX = 0;
        this.rightStick.dirY = 0;
      }
    }
  }

  private rebuildSpatialGrid(): void {
    this.cellHead.fill(-1);
    const n = this.enemyActive;
    for (let i = 0; i < n; i++) {
      const e = this.enemies[i];
      if (!e.alive) continue;
      let cx = Math.floor(e.x / CELL_SIZE) + GRID_HALF;
      let cy = Math.floor(e.y / CELL_SIZE) + GRID_HALF;
      if (cx < 0) cx = 0;
      else if (cx >= GRID_DIM) cx = GRID_DIM - 1;
      if (cy < 0) cy = 0;
      else if (cy >= GRID_DIM) cy = GRID_DIM - 1;
      const cell = cy * GRID_DIM + cx;
      this.enemyNext[i] = this.cellHead[cell];
      this.cellHead[cell] = i;
    }
  }

  private getDifficulty(): DifficultySpec {
    return DIFFICULTIES[this.meta.difficulty] ?? DIFFICULTIES[0];
  }

  private getSelectedShip(): ShipSpec {
    return SHIPS[this.meta.selectedShip] ?? SHIPS[0];
  }

  private startRun(): void {
    this.bulletActive = 0;
    this.eBulletActive = 0;
    this.hazardActive = 0;
    this.mineActive = 0;
    this.enemyActive = 0;
    this.particleActive = 0;
    this.ringActive = 0;
    this.textActive = 0;
    this.gemActive = 0;
    this.arcActive = 0;

    this.leftStick = makeStick();
    this.rightStick = makeStick();

    const shipId = this.meta.selectedShip;
    const diff = this.getDifficulty();
    const ship = this.getSelectedShip();
    const prestigeDmg = 1 + this.meta.prestigeLvl * 0.1;
    const prestigeHp = 1 + this.meta.prestigeLvl * 0.08;
    const prestigeShards = 1 + this.meta.prestigeLvl * 0.1;

    this.playerX = 0;
    this.playerY = 0;
    this.playerVx = 0;
    this.playerVy = 0;
    this.dashTimer = 0;
    this.dashCd = 0;
    this.dashDuration = 0.18 + this.meta.dashLvl * 0.015;
    this.dashMaxCd = Math.max(
      0.42,
      1.6 * Math.pow(0.88, this.meta.dashLvl) * (shipId === 3 ? 0.68 : 1.0)
    );

    this.wave = 1;
    this.waveTimer = 0;
    this.waveEvent = 'NORMAL';
    this.bannerText = `FALA 1 — ${ship.name} [${diff.name}]`;
    this.bannerSub =
      diff.id === 0
        ? 'ZBIERAJ PRZEKAZNIKI I UNIKAJ OGNIA'
        : `MNOZNIK SHARDOW ${diff.shardMult.toFixed(1)}x — EKSTREMALNE ZAGROZENIE`;
    this.bannerTimer = 3.2;
    this.bannerRgb = diff.rgb;
    this.hazardSpawnTimer = diff.id === 2 ? 3.2 : 5.0;
    this.beaconSpawnTimer = 14.0;
    this.beacon.active = false;
    this.beacon.progress = 0;
    this.overdriveTimer = 0;
    this.combo = 0;
    this.comboTimer = 0;
    this.hurtFlash = 0;
    this.spawnAccum = 0;
    this.level = 1;
    this.xp = 0;
    this.nextXp = 25;
    this.runShards = 0;
    this.kills = 0;
    this.isNewRecord = false;

    const shipHpMult = shipId === 2 ? 1.7 : shipId === 1 ? 0.85 : 1.0;
    this.maxHp = Math.round((100 + this.meta.hpLvl * 30) * prestigeHp * shipHpMult);
    this.hp = this.maxHp;
    this.maxShield = this.meta.shieldLvl * 35 + (shipId === 2 ? 60 : 0);
    this.shield = this.maxShield;
    this.shieldDelay = 0;
    this.hpRegen = this.meta.regenLvl * 1.5;
    this.critChance = Math.min(0.9, this.meta.critLvl * 0.08 + (shipId === 3 ? 0.15 : 0));
    this.greedMult = (1 + this.meta.greedLvl * 0.2) * prestigeShards * diff.shardMult;

    const shipDmgMult = shipId === 2 ? 1.35 : 1.0;
    this.bulletDmg = 18 * (1 + this.meta.dmgLvl * 0.15) * prestigeDmg * shipDmgMult;
    const shipRofMult = shipId === 1 ? 0.78 : shipId === 2 ? 1.35 : 1.0;
    this.fireInterval = Math.max(0.048, 0.21 * Math.pow(0.91, this.meta.rateLvl) * shipRofMult);
    const shipSpdMult = shipId === 1 ? 1.22 : shipId === 2 ? 0.86 : 1.0;
    this.moveSpeed = 275 * (1 + this.meta.spdLvl * 0.08) * shipSpdMult;

    const shipExtraBullets = shipId === 2 ? 3 : shipId === 1 ? 1 : 0;
    this.bulletCount = 1 + this.meta.startSalvoLvl + shipExtraBullets;
    this.bulletPierce = 1 + Math.floor(this.meta.startSalvoLvl / 2);
    this.bulletSpread = shipId === 2 ? 0.165 : 0.11;
    this.magnetRadius = 130 * (1 + this.meta.magLvl * 0.35);

    this.orbitCount = this.meta.droneLvl;
    this.orbitDmg = 16 + this.meta.droneLvl * 4;
    this.novaLvl = Math.ceil(this.meta.supernovaLvl / 2);
    this.teslaLvl = Math.floor(this.meta.supernovaLvl / 2);
    this.novaTimer = 1.6;
    this.teslaTimer = 1.0;

    for (let i = 0; i < 8 + diff.id * 2; i++) {
      this.spawnEnemy(0);
    }
    this.state = { kind: 'Playing' };
  }

  private triggerDash(): void {
    if (this.dashCd > 0) return;
    this.sounds.play('dash', this.settings.sound, 0.8);
    if (this.meta.selectedShip === 3) {
      // Phantom Wraith signature: drop an armed proximity plasma mine at pre-dash position
      this.spawnMine(this.playerX, this.playerY, this.bulletDmg * 4.5, 138);
    }
    let dx = this.leftStick.dirX;
    let dy = this.leftStick.dirY;
    if (this.keysDown.has('KeyW') || this.keysDown.has('ArrowUp')) dy -= 1;
    if (this.keysDown.has('KeyS') || this.keysDown.has('ArrowDown')) dy += 1;
    if (this.keysDown.has('KeyA') || this.keysDown.has('ArrowLeft')) dx -= 1;
    if (this.keysDown.has('KeyD') || this.keysDown.has('ArrowRight')) dx += 1;

    const lenSq = dx * dx + dy * dy;
    if (lenSq > 0.01) {
      const inv = 1 / Math.sqrt(lenSq);
      this.dashDirX = dx * inv;
      this.dashDirY = dy * inv;
    } else {
      this.dashDirX = Math.cos(this.aimAngle);
      this.dashDirY = Math.sin(this.aimAngle);
    }
    this.dashTimer = this.dashDuration;
    this.dashCd = this.dashMaxCd;
    this.spawnRing(this.playerX, this.playerY, 75, this.getSelectedShip().rgb, 0.25);
  }

  private updatePlaying(dt: number): void {
    if (this.keysPressed.has('Space') || this.keysPressed.has('ShiftLeft')) {
      this.triggerDash();
    }
    if (this.keysPressed.has('Escape')) {
      this.state = { kind: 'Paused' };
      return;
    }

    const diff = this.getDifficulty();

    // Passive hull & plasma shield regeneration
    if (this.hpRegen > 0 && this.hp < this.maxHp) {
      this.hp = Math.min(this.maxHp, this.hp + this.hpRegen * dt);
    }
    if (this.maxShield > 0) {
      if (this.shieldDelay > 0) {
        this.shieldDelay = Math.max(0, this.shieldDelay - dt);
      } else if (this.shield < this.maxShield) {
        const shieldRegen = 18 + this.meta.shieldLvl * 5;
        this.shield = Math.min(this.maxShield, this.shield + shieldRegen * dt);
      }
    }
    if (this.bannerTimer > 0) this.bannerTimer = Math.max(0, this.bannerTimer - dt);
    if (this.hurtFlash > 0) this.hurtFlash = Math.max(0, this.hurtFlash - dt * 2.8);
    if (this.overdriveTimer > 0) this.overdriveTimer = Math.max(0, this.overdriveTimer - dt);
    if (this.comboTimer > 0) {
      this.comboTimer -= dt;
      if (this.comboTimer <= 0) this.combo = 0;
    }

    // Wave progression & dynamic wave events (22s waves)
    this.waveTimer += dt;
    if (this.waveTimer >= 22) {
      this.waveTimer = 0;
      this.wave += 1;
      this.startNextWave();
    }

    // Dynamic Spawner tailored to current wave, active event & difficulty
    const eventRateMult =
      this.waveEvent === 'SWARM' ? 1.55 : this.waveEvent === 'BOMBER' ? 1.25 : 1.0;
    const diffRateMult = 1 + diff.id * 0.22;
    const spawnRate = Math.min(18, (2.0 + this.wave * 0.72) * eventRateMult * diffRateMult);
    this.spawnAccum += dt * spawnRate;
    while (this.spawnAccum >= 1) {
      this.spawnAccum -= 1;
      const r = Math.random();
      let kind = 0;
      if (this.waveEvent === 'SNIPER') {
        kind = r > 0.72 ? (this.wave >= 5 ? 6 : 4) : r > 0.36 ? 4 : r > 0.18 ? 1 : 0;
      } else if (this.waveEvent === 'BOMBER') {
        kind = r > 0.56 ? 5 : r > 0.34 ? 1 : r > 0.22 ? 2 : 0;
      } else if (this.waveEvent === 'SWARM') {
        kind = r > 0.75 ? 1 : r > 0.65 && this.wave >= 4 ? 5 : 0;
      } else {
        if (this.wave >= 5 && r > 0.90) kind = 6;
        else if (this.wave >= 4 && r > 0.80) kind = 5;
        else if (this.wave >= 2 && r > 0.68) kind = 4;
        else if (this.wave >= 3 && r > 0.56) kind = 2;
        else if (this.wave >= 2 && r > 0.38) kind = 1;
      }
      this.spawnEnemy(kind);
    }

    // Orbital Hazard Strikes (Wave 3+ or Wave 2+ on Hardcore/Nightmare)
    if (this.wave >= (diff.id === 2 ? 2 : 3) || this.waveEvent === 'ORBITAL') {
      this.hazardSpawnTimer -= dt;
      if (this.hazardSpawnTimer <= 0) {
        const isOrbital = this.waveEvent === 'ORBITAL';
        const baseCd = isOrbital
          ? Math.max(0.36, (1.05 - this.wave * 0.04) / (1 + diff.id * 0.16))
          : Math.max(1.0, (3.4 - this.wave * 0.14) / (1 + diff.id * 0.16));
        this.hazardSpawnTimer = baseCd;
        const count = isOrbital ? (this.wave >= 7 || diff.id === 2 ? 3 : 2) : 1;
        for (let h = 0; h < count; h++) {
          const lead = h === 0 ? 0.45 : randRange(0.1, 0.85);
          const hx = this.playerX + this.playerVx * lead + randRange(-140, 140);
          const hy = this.playerY + this.playerVy * lead + randRange(-140, 140);
          const hRad = isOrbital ? randRange(66, 86) : randRange(58, 76);
          const warnTime = Math.max(0.84, 1.38 - this.wave * 0.025 - diff.id * 0.08);
          const hDmg = (22 + this.wave * 2.8) * diff.dmgMult;
          this.spawnHazard(hx, hy, hRad, warnTime, hDmg);
        }
      }
    }

    // Interactive Map Objective: Overload Beacon (encourages roaming across the 2800-radius arena)
    if (!this.beacon.active) {
      this.beaconSpawnTimer -= dt;
      if (this.beaconSpawnTimer <= 0) {
        this.spawnBeacon();
      }
    } else {
      this.beacon.ttl -= dt;
      const bdx = this.playerX - this.beacon.x;
      const bdy = this.playerY - this.beacon.y;
      if (bdx * bdx + bdy * bdy <= this.beacon.radius * this.beacon.radius) {
        this.beacon.progress += dt;
        if (this.beacon.progress >= this.beacon.duration) {
          this.completeBeacon();
          if (this.state.kind !== 'Playing') return;
        }
      } else {
        this.beacon.progress = Math.max(0, this.beacon.progress - dt * 0.45);
      }
      if (this.beacon.active && this.beacon.ttl <= 0) {
        this.beacon.active = false;
        this.beaconSpawnTimer = 18;
      }
    }

    // Player physics
    if (this.dashCd > 0) {
      this.dashCd = Math.max(0, this.dashCd - dt);
    }

    const activeMoveSpeed = this.overdriveTimer > 0 ? this.moveSpeed * 1.3 : this.moveSpeed;
    if (this.dashTimer > 0) {
      this.dashTimer -= dt;
      this.playerVx = this.dashDirX * 860;
      this.playerVy = this.dashDirY * 860;
      this.spawnParticle(
        this.playerX,
        this.playerY,
        randRange(-45, 45),
        randRange(-45, 45),
        0.22,
        5,
        this.overdriveTimer > 0 ? RGB_GOLD : RGB_CYAN
      );
    } else {
      let mx = this.leftStick.dirX;
      let my = this.leftStick.dirY;
      if (this.keysDown.has('KeyW') || this.keysDown.has('ArrowUp')) my -= 1;
      if (this.keysDown.has('KeyS') || this.keysDown.has('ArrowDown')) my += 1;
      if (this.keysDown.has('KeyA') || this.keysDown.has('ArrowLeft')) mx -= 1;
      if (this.keysDown.has('KeyD') || this.keysDown.has('ArrowRight')) mx += 1;
      const mLenSq = mx * mx + my * my;
      if (mLenSq > 1) {
        const inv = 1 / Math.sqrt(mLenSq);
        mx *= inv;
        my *= inv;
      }

      const targetVx = mx * activeMoveSpeed;
      const targetVy = my * activeMoveSpeed;
      const accel = 1 - Math.exp(-18 * dt);
      this.playerVx += (targetVx - this.playerVx) * accel;
      this.playerVy += (targetVy - this.playerVy) * accel;

      const speedSq = this.playerVx * this.playerVx + this.playerVy * this.playerVy;
      if (speedSq > 35 * 35) {
        this.thrustAccum += dt * 45;
        while (this.thrustAccum >= 1) {
          this.thrustAccum -= 1;
          const back = Math.atan2(this.playerVy, this.playerVx) + Math.PI + randRange(-0.25, 0.25);
          const dx = Math.cos(back);
          const dy = Math.sin(back);
          this.spawnParticle(
            this.playerX + dx * 12,
            this.playerY + dy * 12,
            dx * 120,
            dy * 120,
            0.2,
            3,
            this.overdriveTimer > 0 ? RGB_GOLD : RGB_CYAN
          );
        }
      }
    }

    this.playerX += this.playerVx * dt;
    this.playerY += this.playerVy * dt;
    const maxPlayerR = ARENA_RADIUS - 18;
    const distCenterSq = this.playerX * this.playerX + this.playerY * this.playerY;
    if (distCenterSq > maxPlayerR * maxPlayerR) {
      const s = maxPlayerR / Math.sqrt(distCenterSq);
      this.playerX *= s;
      this.playerY *= s;
    }

    // Build spatial hash once per frame for all collision & targeting queries
    this.rebuildSpatialGrid();

    // Aiming
    const stickManual =
      this.rightStick.touchId !== null &&
      this.rightStick.dirX * this.rightStick.dirX + this.rightStick.dirY * this.rightStick.dirY >
        0.01;
    const mouseManual = !this.settings.autoAim && this.pointerDown;
    const manualAim = stickManual || mouseManual;
    const nearestIdx = this.findNearestEnemySpatial(this.playerX, this.playerY, 620);

    if (stickManual) {
      this.aimAngle = Math.atan2(this.rightStick.dirY, this.rightStick.dirX);
    } else if (!this.settings.autoAim) {
      this.aimAngle = Math.atan2(
        this.pointerPos.y - this.height * 0.5,
        this.pointerPos.x - this.width * 0.5
      );
    } else if (nearestIdx >= 0) {
      const ne = this.enemies[nearestIdx];
      this.aimAngle = Math.atan2(ne.y - this.playerY, ne.x - this.playerX);
    } else if (this.playerVx * this.playerVx + this.playerVy * this.playerVy > 400) {
      this.aimAngle = Math.atan2(this.playerVy, this.playerVx);
    }

    // Primary weapon (Overdrive boosts fire rate & Overdrive Reactor boosts damage)
    this.sounds.shootCd = Math.max(0, this.sounds.shootCd - dt);
    this.fireCd -= dt;
    if (this.fireCd <= 0 && (manualAim || !this.settings.autoAim || nearestIdx >= 0)) {
      this.fireCd = this.overdriveTimer > 0 ? this.fireInterval * 0.68 : this.fireInterval;
      if (this.sounds.shootCd <= 0) {
        this.sounds.play('shoot', this.settings.sound, 0.5);
        this.sounds.shootCd = 0.055;
      }
      const spread = this.bulletSpread;
      const shotDmg =
        this.overdriveTimer > 0
          ? this.bulletDmg * (1 + this.meta.overdriveLvl * 0.12)
          : this.bulletDmg;
      for (let i = 0; i < this.bulletCount; i++) {
        const offset = (i - (this.bulletCount - 1) * 0.5) * spread;
        const a = this.aimAngle + offset;
        const dx = Math.cos(a);
        const dy = Math.sin(a);
        this.spawnBullet(
          this.playerX + dx * 18,
          this.playerY + dy * 18,
          dx * 760,
          dy * 760,
          shotDmg,
          this.bulletPierce
        );
      }
    }

    // Orbital saws (spatial query)
    if (this.orbitCount > 0) {
      this.orbitAngle += dt * 3.6;
      for (let i = 0; i < this.orbitCount; i++) {
        const a = this.orbitAngle + (i * TAU) / this.orbitCount;
        const bx = this.playerX + Math.cos(a) * 74;
        const by = this.playerY + Math.sin(a) * 74;
        const cx0 = Math.max(0, Math.floor((bx - 52) / CELL_SIZE) + GRID_HALF);
        const cx1 = Math.min(GRID_DIM - 1, Math.floor((bx + 52) / CELL_SIZE) + GRID_HALF);
        const cy0 = Math.max(0, Math.floor((by - 52) / CELL_SIZE) + GRID_HALF);
        const cy1 = Math.min(GRID_DIM - 1, Math.floor((by + 52) / CELL_SIZE) + GRID_HALF);
        for (let cy = cy0; cy <= cy1; cy++) {
          const row = cy * GRID_DIM;
          for (let cx = cx0; cx <= cx1; cx++) {
            let j = this.cellHead[row + cx];
            while (j !== -1) {
              const e = this.enemies[j];
              if (e.alive) {
                const dx = e.x - bx;
                const dy = e.y - by;
                const rr = e.radius + 12;
                if (dx * dx + dy * dy < rr * rr) {
                  this.damageEnemy(j, this.orbitDmg * dt * 8, bx, by, false);
                }
              }
              j = this.enemyNext[j];
            }
          }
        }
      }
    }

    // Chain Tesla
    if (this.teslaLvl > 0) {
      this.teslaTimer -= dt;
      if (this.teslaTimer <= 0) {
        this.teslaTimer = Math.max(0.45, 1.4 - this.teslaLvl * 0.18);
        this.triggerTesla();
      }
    }

    // EMP Nova
    if (this.novaLvl > 0) {
      this.novaTimer -= dt;
      if (this.novaTimer <= 0) {
        this.novaTimer = Math.max(1.8, 4.5 - this.novaLvl * 0.45);
        this.triggerNova();
      }
    }

    // Update bullets (dense swap-remove + spatial hash collision)
    const arenaR2 = ARENA_RADIUS * ARENA_RADIUS;
    for (let i = this.bulletActive - 1; i >= 0; i--) {
      const b = this.bullets[i];
      b.x += b.vx * dt;
      b.y += b.vy * dt;
      b.life -= dt;
      if (b.life <= 0 || b.x * b.x + b.y * b.y > arenaR2) {
        this.removeBullet(i);
        continue;
      }

      const cx0 = Math.max(0, Math.floor((b.x - 45) / CELL_SIZE) + GRID_HALF);
      const cx1 = Math.min(GRID_DIM - 1, Math.floor((b.x + 45) / CELL_SIZE) + GRID_HALF);
      const cy0 = Math.max(0, Math.floor((b.y - 45) / CELL_SIZE) + GRID_HALF);
      const cy1 = Math.min(GRID_DIM - 1, Math.floor((b.y + 45) / CELL_SIZE) + GRID_HALF);

      let dead = false;
      for (let cy = cy0; cy <= cy1 && !dead; cy++) {
        const row = cy * GRID_DIM;
        for (let cx = cx0; cx <= cx1 && !dead; cx++) {
          let j = this.cellHead[row + cx];
          while (j !== -1) {
            const e = this.enemies[j];
            if (e.alive) {
              const dx = e.x - b.x;
              const dy = e.y - b.y;
              const rr = e.radius + 7;
              if (dx * dx + dy * dy < rr * rr) {
                this.damageEnemy(j, b.dmg, b.x, b.y, true);
                b.pierce -= 1;
                if (b.pierce <= 0) {
                  this.removeBullet(i);
                  dead = true;
                  break;
                }
              }
            }
            j = this.enemyNext[j];
          }
        }
      }
    }

    // Update Phantom Wraith proximity plasma mines
    for (let i = this.mineActive - 1; i >= 0; i--) {
      const m = this.mines[i];
      m.life -= dt;
      const near = this.findNearestEnemySpatial(m.x, m.y, 62);
      if (near >= 0 || m.life <= 0) {
        this.detonateMine(i);
      }
    }

    // Update enemies (7 distinct AI archetypes + boss bullet-hell + elites) & compact in O(E)
    let writeIdx = 0;
    const eDmgScale = (1 + (this.wave - 1) * 0.14) * diff.dmgMult;
    for (let i = 0; i < this.enemyActive; i++) {
      const e = this.enemies[i];
      if (!e.alive) continue;
      if (writeIdx !== i) {
        const tmp = this.enemies[writeIdx];
        this.enemies[writeIdx] = e;
        this.enemies[i] = tmp;
      }
      writeIdx++;

      if (e.flash > 0) e.flash -= dt;
      const toPx = this.playerX - e.x;
      const toPy = this.playerY - e.y;
      const distSq = toPx * toPx + toPy * toPy;
      const dist = distSq > 0.000001 ? Math.sqrt(distSq) : 0.001;
      const invDist = 1 / dist;
      const dirX = toPx * invDist;
      const dirY = toPy * invDist;
      e.angle += e.spin * dt;
      e.timer += dt;

      if (e.kind === 1) {
        // Dasher: rhythmic high-speed bursts
        const mult = e.timer % 2.1 > 1.55 ? 3.25 : 0.65;
        const step = e.speed * mult * dt;
        e.x += dirX * step;
        e.y += dirY * step;
      } else if (e.kind === 4) {
        // Sniper Drone: kites at ~320px, strafes, telegraphs red laser, fires aimed crimson plasma
        const strafeDir = (i & 1) === 0 ? 1 : -1;
        const radial = dist > 360 ? 1 : dist < 245 ? -0.85 : 0.05;
        const step = e.speed * dt;
        e.x += (dirX * radial - dirY * 0.72 * strafeDir) * step;
        e.y += (dirY * radial + dirX * 0.72 * strafeDir) * step;

        e.shootCd -= dt;
        if (e.shootCd <= 0 && dist < 680) {
          e.shootCd = Math.max(1.35, 2.35 - this.wave * 0.06);
          const baseA = Math.atan2(toPy, toPx);
          const shotSpd = 460 + Math.min(180, this.wave * 14);
          const shotDmg = (e.elite ? 18 : 13) * eDmgScale;
          if (e.elite) {
            for (let s = -1; s <= 1; s++) {
              const a = baseA + s * 0.16;
              this.spawnEnemyBullet(e.x, e.y, Math.cos(a) * shotSpd, Math.sin(a) * shotSpd, shotDmg, 5.5);
            }
          } else {
            this.spawnEnemyBullet(e.x, e.y, Math.cos(baseA) * shotSpd, Math.sin(baseA) * shotSpd, shotDmg, 5);
          }
          this.sounds.play('shoot', this.settings.sound, 0.35);
        }
      } else if (e.kind === 5) {
        // Shrapnel Bomber: rushes player and detonates into radial enemy bullets when close
        const step = e.speed * dt;
        e.x += dirX * step;
        e.y += dirY * step;
        if (dist < 68) {
          e.alive = false;
          this.spawnRing(e.x, e.y, 115, RGB_LIME, 0.32);
          this.sounds.play('boom', this.settings.sound, 0.75);
          const pellets = e.elite ? 10 : 7;
          const bSpd = 250 + Math.min(120, this.wave * 8);
          const bDmg = 13 * eDmgScale;
          for (let p = 0; p < pellets; p++) {
            const a = e.angle + (p * TAU) / pellets;
            this.spawnEnemyBullet(e.x, e.y, Math.cos(a) * bSpd, Math.sin(a) * bSpd, bDmg, 5);
          }
          if (this.dashTimer <= 0 && dist < 82) {
            if (this.damagePlayer(20 * eDmgScale, 9)) {
              this.enemyActive = writeIdx;
              return;
            }
          }
          continue;
        }
      } else if (e.kind === 6) {
        // Vortex Elite: orbits at ~255px, gently pulls player inward when close, fires spiral volleys
        const strafeDir = (i & 1) === 0 ? 1 : -1;
        const radial = dist > 290 ? 0.9 : dist < 195 ? -0.6 : 0;
        const step = e.speed * dt;
        e.x += (dirX * radial - dirY * 0.85 * strafeDir) * step;
        e.y += (dirY * radial + dirX * 0.85 * strafeDir) * step;

        if (dist < 270 && this.dashTimer <= 0) {
          const pull = (1 - dist / 270) * (e.elite ? 115 : 80) * dt;
          this.playerX -= dirX * pull;
          this.playerY -= dirY * pull;
        }
        e.shootCd -= dt;
        if (e.shootCd <= 0 && dist < 640) {
          e.shootCd = Math.max(1.45, 2.5 - this.wave * 0.06);
          const count = e.elite ? 6 : 4;
          const bSpd = 220 + Math.min(110, this.wave * 8);
          const bDmg = 12 * eDmgScale;
          for (let s = 0; s < count; s++) {
            const a = e.angle + (s * TAU) / count;
            this.spawnEnemyBullet(e.x, e.y, Math.cos(a) * bSpd, Math.sin(a) * bSpd, bDmg, 5.2);
          }
        }
      } else if (e.kind === 3) {
        // Overlord Boss: steady pursuit + rotating radial bullet-hell rings + enraged <50% HP phase
        const enraged = e.hp < e.maxHp * 0.5;
        const step = e.speed * (enraged ? 1.32 : 1.0) * dt;
        e.x += dirX * step;
        e.y += dirY * step;

        e.shootCd -= dt;
        if (e.shootCd <= 0 && dist < 760) {
          e.shootCd = enraged ? Math.max(0.85, 1.35 - this.wave * 0.025) : Math.max(1.25, 2.0 - this.wave * 0.03);
          const bulletsInRing = Math.min(16, (enraged ? 12 : 8) + Math.floor(this.wave / 3));
          const bSpd = (enraged ? 275 : 225) + Math.min(120, this.wave * 9);
          const bDmg = 15 * eDmgScale;
          for (let s = 0; s < bulletsInRing; s++) {
            const a = e.angle + (s * TAU) / bulletsInRing;
            this.spawnEnemyBullet(e.x, e.y, Math.cos(a) * bSpd, Math.sin(a) * bSpd, bDmg, 6);
          }
          if (enraged) {
            // Aimed pressure shot directly at player during enrage
            const aimA = Math.atan2(toPy, toPx);
            this.spawnEnemyBullet(e.x, e.y, Math.cos(aimA) * (bSpd + 140), Math.sin(aimA) * (bSpd + 140), bDmg * 1.15, 6.5);
          }
          this.spawnRing(e.x, e.y, e.radius * 2.1, enraged ? RGB_CRIMSON : RGB_PURPLE, 0.28);
        }
      } else {
        // Swarmer (0) & Tank (2)
        const step = e.speed * dt;
        e.x += dirX * step;
        e.y += dirY * step;
      }

      if (this.dashTimer <= 0 && dist < e.radius + 14) {
        const dps = (e.kind === 3 ? 78 : e.kind === 2 ? 52 : 42) * (e.elite ? 1.3 : 1);
        if (this.damagePlayer(dps * dt, 5)) {
          this.enemyActive = writeIdx;
          return;
        }
      }
    }
    this.enemyActive = writeIdx;

    // Update enemy bullets (dense swap-remove + Dash i-frame dodging)
    for (let i = this.eBulletActive - 1; i >= 0; i--) {
      const eb = this.eBullets[i];
      eb.x += eb.vx * dt;
      eb.y += eb.vy * dt;
      eb.life -= dt;
      if (eb.life <= 0 || eb.x * eb.x + eb.y * eb.y > arenaR2) {
        this.removeEnemyBullet(i);
        continue;
      }
      if (this.dashTimer <= 0) {
        const dx = eb.x - this.playerX;
        const dy = eb.y - this.playerY;
        const hitR = eb.radius + 10.5;
        if (dx * dx + dy * dy < hitR * hitR) {
          const dmg = eb.dmg;
          this.removeEnemyBullet(i);
          this.spawnParticle(this.playerX, this.playerY, randRange(-90, 90), randRange(-90, 90), 0.22, 4, RGB_CRIMSON);
          if (this.damagePlayer(dmg, 7)) {
            return;
          }
        }
      }
    }

    // Update Orbital Hazard Strikes
    for (let i = this.hazardActive - 1; i >= 0; i--) {
      const h = this.hazards[i];
      h.timer -= dt;
      if (h.timer <= 0) {
        const hx = h.x;
        const hy = h.y;
        const hRad = h.radius;
        const hDmg = h.dmg;
        this.removeHazard(i);
        this.spawnRing(hx, hy, hRad * 1.15, RGB_CRIMSON, 0.34);
        this.sounds.play('boom', this.settings.sound, 0.55);
        for (let p = 0; p < 10; p++) {
          const a = randRange(0, TAU);
          const spd = randRange(60, 210);
          this.spawnParticle(hx, hy, Math.cos(a) * spd, Math.sin(a) * spd, randRange(0.2, 0.42), 3.5, RGB_CRIMSON);
        }
        if (this.dashTimer <= 0) {
          const dx = this.playerX - hx;
          const dy = this.playerY - hy;
          if (dx * dx + dy * dy <= (hRad + 8) * (hRad + 8)) {
            if (this.damagePlayer(hDmg, 10)) {
              return;
            }
          }
        }
      }
    }

    // Update gems (dense swap-remove + combo XP multiplier)
    const magR2 = this.magnetRadius * this.magnetRadius;
    const pickR2 = 24 * 24;
    const comboMult = 1 + Math.min(1.0, this.combo * 0.025);
    for (let i = this.gemActive - 1; i >= 0; i--) {
      const g = this.gems[i];
      const dx = this.playerX - g.x;
      const dy = this.playerY - g.y;
      const distSq = dx * dx + dy * dy;
      if (distSq < magR2 && distSq > 0.01) {
        const inv = (510 * dt) / Math.sqrt(distSq);
        g.x += dx * inv;
        g.y += dy * inv;
      }
      if (distSq < pickR2) {
        const xpGain = g.xp * this.greedMult * comboMult;
        const shardGain = g.shard > 0 ? Math.max(1, Math.round(g.shard * this.greedMult)) : 0;
        this.removeGem(i);
        this.xp += xpGain;
        this.runShards += shardGain;
        if (this.xp >= this.nextXp) {
          this.xp -= this.nextXp;
          this.level += 1;
          this.nextXp = Math.floor(this.nextXp * 1.32 + 12);
          this.sounds.play('lvlup', this.settings.sound, 0.8);
          this.spawnRing(this.playerX, this.playerY, 150, RGB_LIGHT_CYAN, 0.4);
          this.rollLevelUp();
          return;
        }
      }
    }

    // Update VFX (dense swap-remove)
    for (let i = this.particleActive - 1; i >= 0; i--) {
      const p = this.particles[i];
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.life -= dt;
      if (p.life <= 0) this.removeParticle(i);
    }
    for (let i = this.ringActive - 1; i >= 0; i--) {
      const r = this.rings[i];
      r.life -= dt;
      r.radius = r.maxRadius * (1 - Math.max(0, r.life / r.maxLife));
      if (r.life <= 0) this.removeRing(i);
    }
    for (let i = this.textActive - 1; i >= 0; i--) {
      const t = this.texts[i];
      t.y -= 52 * dt;
      t.life -= dt;
      if (t.life <= 0) this.removeText(i);
    }
    for (let i = this.arcActive - 1; i >= 0; i--) {
      const a = this.arcs[i];
      a.life -= dt;
      if (a.life <= 0) this.removeArc(i);
    }

    if (this.shake > 0) {
      this.shake = Math.max(0, this.shake - dt * 30);
    }
  }

  private triggerTesla(): void {
    let cx = this.playerX;
    let cy = this.playerY;
    const hit = new Set<number>();
    const jumps = 2 + this.teslaLvl;
    for (let k = 0; k < jumps; k++) {
      const best = this.findNearestEnemySpatial(cx, cy, 260, hit);
      if (best >= 0) {
        hit.add(best);
        const tx = this.enemies[best].x;
        const ty = this.enemies[best].y;
        this.spawnArc(cx, cy, tx, ty);
        const dmg = this.bulletDmg * (0.8 + this.teslaLvl * 0.25);
        this.damageEnemy(best, dmg, tx, ty, true);
        cx = tx;
        cy = ty;
      } else {
        break;
      }
    }
  }

  private triggerNova(): void {
    const rad = 145 + this.novaLvl * 28 + this.meta.supernovaLvl * 14;
    this.spawnRing(this.playerX, this.playerY, rad, RGB_PURPLE, 0.35);

    // Supernova EMP clears enemy bullets inside the shockwave
    if (this.meta.supernovaLvl > 0) {
      const clearR = rad + 45;
      const clearR2 = clearR * clearR;
      for (let i = this.eBulletActive - 1; i >= 0; i--) {
        const eb = this.eBullets[i];
        const dx = eb.x - this.playerX;
        const dy = eb.y - this.playerY;
        if (dx * dx + dy * dy <= clearR2) {
          this.removeEnemyBullet(i);
        }
      }
      this.spawnRing(this.playerX, this.playerY, clearR, RGB_LIGHT_CYAN, 0.28);
    }

    const span = rad + 40;
    const cx0 = Math.max(0, Math.floor((this.playerX - span) / CELL_SIZE) + GRID_HALF);
    const cx1 = Math.min(GRID_DIM - 1, Math.floor((this.playerX + span) / CELL_SIZE) + GRID_HALF);
    const cy0 = Math.max(0, Math.floor((this.playerY - span) / CELL_SIZE) + GRID_HALF);
    const cy1 = Math.min(GRID_DIM - 1, Math.floor((this.playerY + span) / CELL_SIZE) + GRID_HALF);

    for (let cy = cy0; cy <= cy1; cy++) {
      const row = cy * GRID_DIM;
      for (let cx = cx0; cx <= cx1; cx++) {
        let i = this.cellHead[row + cx];
        while (i !== -1) {
          const e = this.enemies[i];
          if (e.alive) {
            const dx = e.x - this.playerX;
            const dy = e.y - this.playerY;
            const rr = rad + e.radius;
            const d2 = dx * dx + dy * dy;
            if (d2 < rr * rr) {
              if (d2 > 0.01) {
                const inv = 95 / Math.sqrt(d2);
                e.x += dx * inv;
                e.y += dy * inv;
              }
              const dmg = this.bulletDmg * (1.1 + this.novaLvl * 0.35 + this.meta.supernovaLvl * 0.12);
              this.damageEnemy(i, dmg, e.x, e.y, true);
            }
          }
          i = this.enemyNext[i];
        }
      }
    }
  }

  private damageEnemy(idx: number, rawDmg: number, hitX: number, hitY: number, showNum: boolean): void {
    const e = this.enemies[idx];
    const isCrit = showNum && this.critChance > 0 && Math.random() < this.critChance;
    const dmg = isCrit ? rawDmg * 2 : rawDmg;

    e.hp -= dmg;
    e.flash = 0.06;
    if (showNum && this.settings.dmgNums) {
      const col = isCrit || dmg >= 35 ? RGB_GOLD : RGB_WHITE;
      const label = isCrit ? `${Math.ceil(dmg)}!` : `${Math.ceil(dmg)}`;
      this.spawnText(e.x + randRange(-8, 8), e.y - e.radius, label, col);
    }

    if (e.hp <= 0 && e.alive) {
      e.alive = false;
      this.kills += 1;
      this.combo += 1;
      this.comboTimer = 3.2;
      const kind = e.kind;
      const elite = e.elite;
      const ex = e.x;
      const ey = e.y;
      const erad = e.radius;
      const eAng = e.angle;

      this.sounds.play('boom', this.settings.sound, kind === 3 ? 1 : elite ? 0.8 : 0.6);
      if (this.settings.shake) {
        this.shake = Math.max(this.shake, kind === 3 ? 12 : elite ? 6.5 : 3.5);
      }

      const col =
        kind === 1
          ? RGB_ORANGE
          : kind === 2
            ? RGB_GOLD
            : kind === 3
              ? RGB_PURPLE
              : kind === 4
                ? RGB_CRIMSON
                : kind === 5
                  ? RGB_LIME
                  : kind === 6
                    ? RGB_INDIGO
                    : RGB_PINK;
      this.spawnRing(ex, ey, erad * (elite ? 3.2 : 2.6), col, 0.26);

      const count = kind === 3 ? 28 : elite ? 15 : 9;
      for (let k = 0; k < count; k++) {
        const a = randRange(0, TAU);
        const spd = randRange(70, 290);
        this.spawnParticle(ex, ey, Math.cos(a) * spd, Math.sin(a) * spd, randRange(0.25, 0.5), 3.5, col);
      }

      if (kind === 3) {
        for (let k = 0; k < 4; k++) {
          this.spawnEnemyAt(0, ex + randRange(-26, 26), ey + randRange(-26, 26), false);
        }
        if (this.meta.overdriveLvl > 0) {
          this.overdriveTimer = Math.max(this.overdriveTimer, 3.5 + this.meta.overdriveLvl * 0.45);
          this.spawnText(ex, ey - 24, 'BOSS OVERDRIVE!', RGB_GOLD);
        }
      } else if (kind === 5) {
        // Shrapnel Bomber always bursts enemy shrapnel on death!
        const pellets = elite ? 8 : 6;
        const bSpd = 230 + Math.min(110, this.wave * 8);
        const bDmg = 11 * (1 + (this.wave - 1) * 0.14) * this.getDifficulty().dmgMult;
        for (let p = 0; p < pellets; p++) {
          const a = eAng + (p * TAU) / pellets;
          this.spawnEnemyBullet(ex, ey, Math.cos(a) * bSpd, Math.sin(a) * bSpd, bDmg, 4.8);
        }
      }

      let shard =
        kind === 3
          ? 14
          : kind === 6
            ? 5
            : kind === 2 || kind === 4 || kind === 5
              ? 3
              : Math.random() < 0.32
                ? 1
                : 0;
      let xp = kind === 3 ? 55 : kind === 6 ? 24 : kind === 2 ? 16 : kind === 4 || kind === 5 ? 14 : 6;
      if (elite) {
        shard = Math.max(3, shard * 3);
        xp = Math.round(xp * 2.4);
      }
      this.spawnGem(ex, ey, xp, shard);
    } else {
      this.sounds.play('hit', this.settings.sound, 0.35);
      this.spawnParticle(hitX, hitY, randRange(-70, 70), randRange(-70, 70), 0.14, 2.5, RGB_WHITE);
    }
  }

  private rollLevelUp(): void {
    const pool = [...ALL_UPGRADES];
    for (let i = pool.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [pool[i], pool[j]] = [pool[j], pool[i]];
    }
    this.levelChoices = [pool[0], pool[1], pool[2]];
    this.state = { kind: 'LevelUp' };
  }

  private applyUpgrade(id: number): void {
    switch (id) {
      case 0:
        this.bulletCount += 1;
        break;
      case 1:
        this.bulletDmg *= 1.25;
        break;
      case 2:
        this.fireInterval = Math.max(0.045, this.fireInterval * 0.83);
        break;
      case 3:
        this.bulletPierce += 1;
        break;
      case 4:
        this.orbitCount += 1;
        this.orbitDmg += 6;
        break;
      case 5:
        this.teslaLvl += 1;
        break;
      case 6:
        this.novaLvl += 1;
        break;
      case 7:
        this.dashMaxCd = Math.max(0.45, this.dashMaxCd * 0.75);
        break;
      case 8:
        this.maxHp += 25;
        this.hp = Math.min(this.maxHp, this.hp + this.maxHp * 0.4);
        break;
      case 9:
        this.magnetRadius *= 1.45;
        break;
      case 10:
        this.critChance = Math.min(0.95, this.critChance + 0.15);
        break;
      case 11:
        this.hpRegen += 3;
        break;
    }
    this.state = { kind: 'Playing' };
  }

  private finishRun(victory: boolean): void {
    this.meta.shards += this.runShards;
    this.isNewRecord = this.wave > this.meta.bestWave || this.kills > this.meta.bestKills;
    if (this.wave > this.meta.bestWave) this.meta.bestWave = this.wave;
    if (this.kills > this.meta.bestKills) this.meta.bestKills = this.kills;
    saveMeta(this.meta);
    this.state = { kind: 'GameOver', victory };
  }

  private findNearestEnemySpatial(
    ox: number,
    oy: number,
    maxRange: number,
    exclude?: Set<number>
  ): number {
    let best = -1;
    let bestD2 = maxRange * maxRange;
    const cx0 = Math.max(0, Math.floor((ox - maxRange) / CELL_SIZE) + GRID_HALF);
    const cx1 = Math.min(GRID_DIM - 1, Math.floor((ox + maxRange) / CELL_SIZE) + GRID_HALF);
    const cy0 = Math.max(0, Math.floor((oy - maxRange) / CELL_SIZE) + GRID_HALF);
    const cy1 = Math.min(GRID_DIM - 1, Math.floor((oy + maxRange) / CELL_SIZE) + GRID_HALF);

    for (let cy = cy0; cy <= cy1; cy++) {
      const row = cy * GRID_DIM;
      for (let cx = cx0; cx <= cx1; cx++) {
        let i = this.cellHead[row + cx];
        while (i !== -1) {
          const e = this.enemies[i];
          if (e.alive && (!exclude || !exclude.has(i))) {
            const dx = e.x - ox;
            const dy = e.y - oy;
            const d2 = dx * dx + dy * dy;
            if (d2 < bestD2) {
              bestD2 = d2;
              best = i;
            }
          }
          i = this.enemyNext[i];
        }
      }
    }
    return best;
  }

  private spawnBullet(x: number, y: number, vx: number, vy: number, dmg: number, pierce: number): void {
    if (this.bulletActive >= MAX_BULLETS) return;
    const b = this.bullets[this.bulletActive++];
    b.x = x;
    b.y = y;
    b.vx = vx;
    b.vy = vy;
    b.dmg = dmg;
    b.life = 1.15;
    b.pierce = pierce;
  }

  private removeBullet(idx: number): void {
    const last = --this.bulletActive;
    if (idx !== last) {
      const tmp = this.bullets[idx];
      this.bullets[idx] = this.bullets[last];
      this.bullets[last] = tmp;
    }
  }

  private damagePlayer(amount: number, shakeAmt: number): boolean {
    let rem = amount;
    if (this.maxShield > 0) {
      this.shieldDelay = 3.8;
      if (this.shield > 0) {
        const absorbed = Math.min(this.shield, rem);
        this.shield -= absorbed;
        rem -= absorbed;
        if (absorbed >= 3) {
          this.spawnRing(this.playerX, this.playerY, 34, RGB_LIGHT_CYAN, 0.18);
        }
      }
    }
    if (rem <= 0) {
      if (this.settings.shake) {
        this.shake = Math.max(this.shake, shakeAmt * 0.45);
      }
      return false;
    }

    this.hp -= rem;
    this.hurtFlash = Math.min(1, this.hurtFlash + 0.45);
    if (this.settings.shake) {
      this.shake = Math.max(this.shake, shakeAmt);
    }
    if (this.hp <= 0) {
      this.hp = 0;
      this.finishRun(false);
      return true;
    }
    return false;
  }

  private startNextWave(): void {
    const w = this.wave;
    const diff = this.getDifficulty();
    if (w % 3 === 0) {
      this.waveEvent = 'BOSS';
      const bossCount =
        w >= diff.doubleBossWave + 9
          ? 4
          : w >= diff.doubleBossWave + 6
            ? 3
            : w >= diff.doubleBossWave
              ? 2
              : 1;
      this.bannerText = `FALA ${w} — ALARM: OVERLORD BOSS x${bossCount}!`;
      this.bannerSub =
        bossCount >= 2
          ? 'WIELU BOSSOW + ESKORTA SNAJPEROW'
          : 'UNIKAJ FAL POCISKOW — DASH DAJE NIESMIERTELNOSC';
      this.bannerRgb = RGB_PURPLE;
      for (let b = 0; b < bossCount; b++) {
        this.spawnEnemy(3);
      }
      for (let s = 0; s < 2 + Math.floor(w / 4) + diff.id; s++) {
        this.spawnEnemy(4);
      }
    } else {
      const cycle: WaveEventKind[] =
        w >= 4 ? ['SWARM', 'ORBITAL', 'SNIPER', 'BOMBER'] : ['SNIPER', 'ORBITAL', 'SWARM'];
      this.waveEvent = cycle[w % cycle.length];
      if (this.waveEvent === 'SWARM') {
        this.bannerText = `FALA ${w} — ROJ ZASADZKI!`;
        this.bannerSub = 'OTACZAJA CIE SZYBKIE JEDNOSTKI';
        this.bannerRgb = RGB_ORANGE;
        const ringCount = Math.min(20, 9 + w);
        for (let k = 0; k < ringCount; k++) {
          const a = (k * TAU) / ringCount;
          const ex = this.playerX + Math.cos(a) * 430;
          const ey = this.playerY + Math.sin(a) * 430;
          const lim = ARENA_RADIUS - 50;
          const d2 = ex * ex + ey * ey;
          const s = d2 > lim * lim ? lim / Math.sqrt(d2) : 1;
          this.spawnEnemyAt(k % 3 === 0 ? 1 : 0, ex * s, ey * s);
        }
      } else if (this.waveEvent === 'ORBITAL') {
        this.bannerText = `FALA ${w} — OSTRZAL ORBITALNY!`;
        this.bannerSub = 'UCIEKAJ Z CZERWONYCH STREF UDERZENIA';
        this.bannerRgb = RGB_CRIMSON;
        this.hazardSpawnTimer = 0.55;
      } else if (this.waveEvent === 'SNIPER') {
        this.bannerText = `FALA ${w} — ESKADRA SNAJPEROW!`;
        this.bannerSub = 'UWAGA NA LASERY NAMIERZAJACE I WIRY';
        this.bannerRgb = RGB_INDIGO;
        for (let k = 0; k < 4 + Math.floor(w / 2); k++) {
          this.spawnEnemy(k % 3 === 0 && w >= 5 ? 6 : 4);
        }
      } else {
        this.bannerText = `FALA ${w} — FALA BOMBOWCOW!`;
        this.bannerSub = 'NISZCZ Z DYSTANSU — EKSPLODUJA ODŁAMKAMI';
        this.bannerRgb = RGB_LIME;
        for (let k = 0; k < 5 + Math.floor(w / 2); k++) {
          this.spawnEnemy(5);
        }
      }
    }
    this.bannerTimer = 3.6;
    this.sounds.play('lvlup', this.settings.sound, 0.55);
  }

  private spawnBeacon(): void {
    const a = randRange(0, TAU);
    const dist = randRange(540, 1020);
    let bx = this.playerX + Math.cos(a) * dist;
    let by = this.playerY + Math.sin(a) * dist;
    const limit = ARENA_RADIUS - 260;
    const lenSq = bx * bx + by * by;
    if (lenSq > limit * limit) {
      const s = limit / Math.sqrt(lenSq);
      bx *= s;
      by *= s;
    }
    this.beacon.active = true;
    this.beacon.x = bx;
    this.beacon.y = by;
    this.beacon.radius = 82;
    this.beacon.progress = 0;
    this.beacon.duration = 2.4;
    this.beacon.ttl = 28;
    if (this.bannerTimer <= 0.4) {
      this.bannerText = 'PRZEKAZNIK MOCY AKTYWNY';
      this.bannerSub = 'PRZEJMIJ ZLOTY PUNKT NA RADARZE — OVERDRIVE + SHARDS';
      this.bannerRgb = RGB_GOLD;
      this.bannerTimer = 3.0;
    }
  }

  private completeBeacon(): void {
    const bx = this.beacon.x;
    const by = this.beacon.y;
    this.beacon.active = false;
    this.beacon.progress = 0;
    this.beaconSpawnTimer = 22;
    this.overdriveTimer = Math.max(this.overdriveTimer, 5.5 + this.meta.overdriveLvl * 1.2);

    const bonusShards = Math.round((6 + this.wave * 2) * this.greedMult);
    const bonusXp = (22 + this.wave * 6) * this.greedMult;
    this.runShards += bonusShards;
    this.xp += bonusXp;

    // Clear nearby enemy bullets in a golden EMP burst
    for (let i = this.eBulletActive - 1; i >= 0; i--) {
      const eb = this.eBullets[i];
      const dx = eb.x - bx;
      const dy = eb.y - by;
      if (dx * dx + dy * dy <= 480 * 480) {
        this.removeEnemyBullet(i);
      }
    }

    this.spawnRing(bx, by, 360, RGB_GOLD, 0.5);
    this.spawnText(bx, by - 28, `OVERDRIVE! +${bonusShards} SHARDS`, RGB_GOLD);
    this.sounds.play('lvlup', this.settings.sound, 0.95);
    if (this.settings.shake) this.shake = Math.max(this.shake, 7);

    if (this.xp >= this.nextXp) {
      this.xp -= this.nextXp;
      this.level += 1;
      this.nextXp = Math.floor(this.nextXp * 1.32 + 12);
      this.rollLevelUp();
    }
  }

  private spawnEnemyBullet(
    x: number,
    y: number,
    vx: number,
    vy: number,
    dmg: number,
    radius: number
  ): void {
    if (this.eBulletActive >= MAX_EBULLETS) return;
    const eb = this.eBullets[this.eBulletActive++];
    eb.x = x;
    eb.y = y;
    eb.vx = vx;
    eb.vy = vy;
    eb.dmg = dmg;
    eb.life = 3.6;
    eb.radius = radius;
  }

  private removeEnemyBullet(idx: number): void {
    const last = --this.eBulletActive;
    if (idx !== last) {
      const tmp = this.eBullets[idx];
      this.eBullets[idx] = this.eBullets[last];
      this.eBullets[last] = tmp;
    }
  }

  private spawnHazard(x: number, y: number, radius: number, maxTimer: number, dmg: number): void {
    if (this.hazardActive >= MAX_HAZARDS) return;
    const lim = ARENA_RADIUS - radius - 20;
    const d2 = x * x + y * y;
    if (d2 > lim * lim) {
      const s = lim / Math.sqrt(d2);
      x *= s;
      y *= s;
    }
    const h = this.hazards[this.hazardActive++];
    h.x = x;
    h.y = y;
    h.radius = radius;
    h.timer = maxTimer;
    h.maxTimer = maxTimer;
    h.dmg = dmg;
  }

  private removeHazard(idx: number): void {
    const last = --this.hazardActive;
    if (idx !== last) {
      const tmp = this.hazards[idx];
      this.hazards[idx] = this.hazards[last];
      this.hazards[last] = tmp;
    }
  }

  private spawnMine(x: number, y: number, dmg: number, radius: number): void {
    if (this.mineActive >= MAX_MINES) {
      this.detonateMine(0);
    }
    const m = this.mines[this.mineActive++];
    m.x = x;
    m.y = y;
    m.life = 9.0;
    m.dmg = dmg;
    m.radius = radius;
  }

  private removeMine(idx: number): void {
    const last = --this.mineActive;
    if (idx !== last) {
      const tmp = this.mines[idx];
      this.mines[idx] = this.mines[last];
      this.mines[last] = tmp;
    }
  }

  private detonateMine(idx: number): void {
    const m = this.mines[idx];
    const mx = m.x;
    const my = m.y;
    const dmg = m.dmg;
    const rad = m.radius;
    this.removeMine(idx);

    this.spawnRing(mx, my, rad, RGB_PURPLE, 0.34);
    this.sounds.play('boom', this.settings.sound, 0.7);
    for (let k = 0; k < 12; k++) {
      const a = randRange(0, TAU);
      const spd = randRange(80, 260);
      this.spawnParticle(mx, my, Math.cos(a) * spd, Math.sin(a) * spd, 0.32, 3.5, RGB_PURPLE);
    }

    // Blast nearby enemies via spatial grid
    const span = rad + 42;
    const cx0 = Math.max(0, Math.floor((mx - span) / CELL_SIZE) + GRID_HALF);
    const cx1 = Math.min(GRID_DIM - 1, Math.floor((mx + span) / CELL_SIZE) + GRID_HALF);
    const cy0 = Math.max(0, Math.floor((my - span) / CELL_SIZE) + GRID_HALF);
    const cy1 = Math.min(GRID_DIM - 1, Math.floor((my + span) / CELL_SIZE) + GRID_HALF);
    for (let cy = cy0; cy <= cy1; cy++) {
      const row = cy * GRID_DIM;
      for (let cx = cx0; cx <= cx1; cx++) {
        let j = this.cellHead[row + cx];
        while (j !== -1) {
          const e = this.enemies[j];
          if (e.alive) {
            const dx = e.x - mx;
            const dy = e.y - my;
            const rr = rad + e.radius;
            if (dx * dx + dy * dy <= rr * rr) {
              this.damageEnemy(j, dmg, mx, my, true);
            }
          }
          j = this.enemyNext[j];
        }
      }
    }

    // Vaporize enemy bullets caught in the plasma mine blast
    const clearR2 = (rad + 18) * (rad + 18);
    for (let i = this.eBulletActive - 1; i >= 0; i--) {
      const eb = this.eBullets[i];
      const dx = eb.x - mx;
      const dy = eb.y - my;
      if (dx * dx + dy * dy <= clearR2) {
        this.removeEnemyBullet(i);
      }
    }
  }

  private spawnEnemy(kind: number): void {
    const angle = randRange(0, TAU);
    const dist = Math.min(ARENA_RADIUS - 80, Math.max(this.width, this.height) * 0.62 + 80);
    let x = this.playerX + Math.cos(angle) * dist;
    let y = this.playerY + Math.sin(angle) * dist;
    const limit = ARENA_RADIUS - 45;
    const lenSq = x * x + y * y;
    if (lenSq > limit * limit) {
      const s = limit / Math.sqrt(lenSq);
      x *= s;
      y *= s;
    }
    this.spawnEnemyAt(kind, x, y);
  }

  private spawnEnemyAt(kind: number, x: number, y: number, allowElite = true): void {
    if (this.enemyActive >= MAX_ENEMIES) return;
    const e = this.enemies[this.enemyActive++];
    const diff = this.getDifficulty();
    // Super-linear late-game HP scaling + difficulty multiplier
    const hpScale =
      (1 + (this.wave - 1) * 0.30 + (this.wave > 4 ? Math.pow(this.wave - 4, 1.5) * 0.26 : 0)) *
      diff.hpMult;
    const spdScale = (1 + Math.min(0.65, (this.wave - 1) * 0.045)) * diff.spdMult;
    const eliteChance =
      this.wave >= diff.eliteWave
        ? Math.min(0.45, 0.10 + (this.wave - diff.eliteWave) * 0.03 + diff.id * 0.06)
        : 0;
    const elite = allowElite && kind !== 3 && Math.random() < eliteChance;

    e.alive = true;
    e.kind = kind;
    e.elite = elite;
    e.x = x;
    e.y = y;
    e.flash = 0;
    e.timer = randRange(0, 2);
    e.shootCd = randRange(0.9, 1.8);
    e.angle = randRange(0, TAU);
    e.spin = randRange(-3.5, 3.5);

    if (kind === 0) {
      e.maxHp = 24 * hpScale;
      e.speed = 118 * spdScale;
      e.radius = 12;
    } else if (kind === 1) {
      e.maxHp = 36 * hpScale;
      e.speed = 138 * spdScale;
      e.radius = 13;
    } else if (kind === 2) {
      e.maxHp = 135 * hpScale;
      e.speed = 74 * spdScale;
      e.radius = 22;
    } else if (kind === 3) {
      e.maxHp = 640 * hpScale;
      e.speed = 68 * spdScale;
      e.radius = 36;
      e.shootCd = 1.4;
    } else if (kind === 4) {
      // Sniper Drone
      e.maxHp = 48 * hpScale;
      e.speed = 112 * spdScale;
      e.radius = 15;
      e.shootCd = randRange(1.2, 2.0);
    } else if (kind === 5) {
      // Shrapnel Bomber
      e.maxHp = 42 * hpScale;
      e.speed = 158 * spdScale;
      e.radius = 14;
    } else {
      // Vortex Elite (kind === 6)
      e.maxHp = 105 * hpScale;
      e.speed = 92 * spdScale;
      e.radius = 19;
      e.shootCd = randRange(1.1, 1.8);
    }

    if (elite) {
      e.maxHp *= 2.2;
      e.speed *= 1.15;
      e.radius *= 1.26;
    }
    e.hp = e.maxHp;
  }

  private spawnParticle(
    x: number,
    y: number,
    vx: number,
    vy: number,
    life: number,
    size: number,
    rgb: [number, number, number]
  ): void {
    if (this.particleActive >= MAX_PARTICLES) return;
    const p = this.particles[this.particleActive++];
    p.x = x;
    p.y = y;
    p.vx = vx;
    p.vy = vy;
    p.life = life;
    p.maxLife = life;
    p.size = size;
    p.rgb = rgb;
  }

  private removeParticle(idx: number): void {
    const last = --this.particleActive;
    if (idx !== last) {
      const tmp = this.particles[idx];
      this.particles[idx] = this.particles[last];
      this.particles[last] = tmp;
    }
  }

  private spawnRing(
    x: number,
    y: number,
    maxRadius: number,
    rgb: [number, number, number],
    life: number
  ): void {
    if (this.ringActive >= MAX_RINGS) return;
    const r = this.rings[this.ringActive++];
    r.x = x;
    r.y = y;
    r.radius = 4;
    r.maxRadius = maxRadius;
    r.life = life;
    r.maxLife = life;
    r.rgb = rgb;
  }

  private removeRing(idx: number): void {
    const last = --this.ringActive;
    if (idx !== last) {
      const tmp = this.rings[idx];
      this.rings[idx] = this.rings[last];
      this.rings[last] = tmp;
    }
  }

  private spawnText(x: number, y: number, val: string, rgb: [number, number, number]): void {
    if (this.textActive >= MAX_TEXTS) return;
    const t = this.texts[this.textActive++];
    t.x = x;
    t.y = y;
    t.val = val;
    t.life = 0.55;
    t.rgb = rgb;
  }

  private removeText(idx: number): void {
    const last = --this.textActive;
    if (idx !== last) {
      const tmp = this.texts[idx];
      this.texts[idx] = this.texts[last];
      this.texts[last] = tmp;
    }
  }

  private spawnGem(x: number, y: number, xp: number, shard: number): void {
    if (this.gemActive >= MAX_GEMS) return;
    const g = this.gems[this.gemActive++];
    g.x = x;
    g.y = y;
    g.xp = xp;
    g.shard = shard;
  }

  private removeGem(idx: number): void {
    const last = --this.gemActive;
    if (idx !== last) {
      const tmp = this.gems[idx];
      this.gems[idx] = this.gems[last];
      this.gems[last] = tmp;
    }
  }

  private spawnArc(ax: number, ay: number, bx: number, by: number): void {
    if (this.arcActive >= MAX_ARCS) return;
    const a = this.arcs[this.arcActive++];
    a.ax = ax;
    a.ay = ay;
    a.bx = bx;
    a.by = by;
    a.life = 0.14;
  }

  private removeArc(idx: number): void {
    const last = --this.arcActive;
    if (idx !== last) {
      const tmp = this.arcs[idx];
      this.arcs[idx] = this.arcs[last];
      this.arcs[last] = tmp;
    }
  }

  private setFontSize(size: number): void {
    if (this.cachedFontSize !== size) {
      this.cachedFontSize = size;
      this.ctx.font = `700 ${size}px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`;
    }
  }

  private drawCenteredText(text: string, cx: number, y: number, size: number, color: string): void {
    this.setFontSize(size);
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'alphabetic';
    this.ctx.fillStyle = color;
    this.ctx.fillText(text, cx, y);
  }

  private drawText(text: string, x: number, y: number, size: number, color: string): void {
    this.setFontSize(size);
    this.ctx.textAlign = 'left';
    this.ctx.textBaseline = 'alphabetic';
    this.ctx.fillStyle = color;
    this.ctx.fillText(text, x, y);
  }

  private drawRightText(text: string, rx: number, y: number, size: number, color: string): void {
    this.setFontSize(size);
    this.ctx.textAlign = 'right';
    this.ctx.textBaseline = 'alphabetic';
    this.ctx.fillStyle = color;
    this.ctx.fillText(text, rx, y);
  }

  private uiButton(
    rect: Rect,
    label: string,
    sub: string | null,
    borderRgb: [number, number, number],
    filled = false
  ): boolean {
    const ctx = this.ctx;
    const hovered = rectContains(rect, this.pointerPos.x, this.pointerPos.y);
    ctx.fillStyle = filled
      ? rgba(borderRgb, hovered ? 0.28 : 0.18)
      : hovered
        ? 'rgba(18, 30, 56, 0.96)'
        : 'rgba(11, 17, 32, 0.94)';
    ctx.fillRect(rect.x, rect.y, rect.w, rect.h);
    ctx.strokeStyle = rgba(borderRgb, 1);
    ctx.lineWidth = 2;
    ctx.strokeRect(rect.x, rect.y, rect.w, rect.h);

    if (sub !== null) {
      const titleSize = rect.h < 50 ? 12 : 14;
      const subSize = rect.h < 50 ? 10 : 11;
      this.drawCenteredText(label, rect.x + rect.w * 0.5, rect.y + rect.h * 0.44, titleSize, rgba(borderRgb, 1));
      this.drawCenteredText(sub, rect.x + rect.w * 0.5, rect.y + rect.h * 0.8, subSize, '#cbd5e1');
    } else {
      const fSize = rect.h < 40 ? 13 : 16;
      this.drawCenteredText(label, rect.x + rect.w * 0.5, rect.y + rect.h * 0.5 + fSize * 0.35, fSize, rgba(borderRgb, 1));
    }

    if (this.clickPos && rectContains(rect, this.clickPos.x, this.clickPos.y)) {
      this.clickPos = null;
      return true;
    }
    return false;
  }

  private drawPolyLines(
    x: number,
    y: number,
    sides: number,
    radius: number,
    rotRad: number,
    lineWidth: number,
    color: string
  ): void {
    const ctx = this.ctx;
    ctx.beginPath();
    for (let i = 0; i <= sides; i++) {
      const a = rotRad + (i * TAU) / sides;
      const px = x + Math.cos(a) * radius;
      const py = y + Math.sin(a) * radius;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;
    ctx.stroke();
  }

  private renderWorld(): void {
    const ctx = this.ctx;
    const sw = this.width;
    const sh = this.height;
    const sx = this.shake > 0 ? randRange(-this.shake, this.shake) : 0;
    const sy = this.shake > 0 ? randRange(-this.shake, this.shake) : 0;

    // Parallax starfield
    ctx.fillStyle = 'rgba(140, 210, 255, 0.43)';
    for (let i = 0; i < this.stars.length; i++) {
      const s = this.stars[i];
      const x = remEuclid(s.x - this.playerX * s.z, 3200) - 800;
      const y = remEuclid(s.y - this.playerY * s.z, 3200) - 800;
      if (x >= -5 && x <= sw + 5 && y >= -5 && y <= sh + 5) {
        ctx.fillRect(x, y, s.size, s.size);
      }
    }

    const camX = this.playerX - sw * 0.5 + sx;
    const camY = this.playerY - sh * 0.5 + sy;
    const viewMinX = camX - 60;
    const viewMaxX = camX + sw + 60;
    const viewMinY = camY - 60;
    const viewMaxY = camY + sh + 60;

    ctx.save();
    ctx.translate(-camX, -camY);

    // Cyber grid (only visible lines)
    const grid = 80;
    const startX = Math.max(-ARENA_RADIUS, Math.floor(camX / grid) * grid);
    const endX = Math.min(ARENA_RADIUS, camX + sw);
    const startY = Math.max(-ARENA_RADIUS, Math.floor(camY / grid) * grid);
    const endY = Math.min(ARENA_RADIUS, camY + sh);

    ctx.strokeStyle = 'rgba(5, 217, 232, 0.08)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let gx = startX; gx <= endX; gx += grid) {
      ctx.moveTo(gx, startY);
      ctx.lineTo(gx, endY);
    }
    for (let gy = startY; gy <= endY; gy += grid) {
      ctx.moveTo(startX, gy);
      ctx.lineTo(endX, gy);
    }
    ctx.stroke();

    // Sector landmark rings on enlarged map (950, 1900)
    ctx.strokeStyle = 'rgba(5, 217, 232, 0.12)';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(0, 0, 950, 0, TAU);
    ctx.moveTo(1900, 0);
    ctx.arc(0, 0, 1900, 0, TAU);
    ctx.stroke();

    // Arena outer wall
    ctx.beginPath();
    ctx.arc(0, 0, ARENA_RADIUS, 0, TAU);
    ctx.strokeStyle = 'rgba(255, 42, 109, 0.22)';
    ctx.lineWidth = 14;
    ctx.stroke();
    ctx.strokeStyle = rgba(RGB_PINK, 1);
    ctx.lineWidth = 2.5;
    ctx.stroke();

    // Overload Beacon capture zone (world objective)
    if (this.beacon.active) {
      const b = this.beacon;
      const inView =
        b.x + b.radius >= viewMinX &&
        b.x - b.radius <= viewMaxX &&
        b.y + b.radius >= viewMinY &&
        b.y - b.radius <= viewMaxY;
      if (inView) {
        const prog = Math.max(0, Math.min(1, b.progress / b.duration));
        ctx.beginPath();
        ctx.arc(b.x, b.y, b.radius, 0, TAU);
        ctx.fillStyle = 'rgba(249, 200, 14, 0.08)';
        ctx.fill();
        ctx.strokeStyle = 'rgba(249, 200, 14, 0.55)';
        ctx.lineWidth = 2;
        ctx.stroke();

        if (prog > 0) {
          ctx.beginPath();
          ctx.arc(b.x, b.y, b.radius - 4, -Math.PI * 0.5, -Math.PI * 0.5 + prog * TAU);
          ctx.strokeStyle = rgba(RGB_GOLD, 1);
          ctx.lineWidth = 5;
          ctx.stroke();
        }
        this.drawPolyLines(b.x, b.y, 4, 16, b.ttl * 2.2, 2.5, rgba(RGB_GOLD, 1));
        this.drawCenteredText('PRZEKAZNIK', b.x, b.y - b.radius - 8, 11, rgba(RGB_GOLD, 0.92));
      }
    }

    // Orbital Hazard Strike telegraphs
    for (let i = 0; i < this.hazardActive; i++) {
      const h = this.hazards[i];
      if (
        h.x + h.radius < viewMinX ||
        h.x - h.radius > viewMaxX ||
        h.y + h.radius < viewMinY ||
        h.y - h.radius > viewMaxY
      ) {
        continue;
      }
      const p = 1 - Math.max(0, Math.min(1, h.timer / h.maxTimer));
      ctx.beginPath();
      ctx.arc(h.x, h.y, h.radius, 0, TAU);
      ctx.fillStyle = `rgba(255, 58, 92, ${0.07 + p * 0.14})`;
      ctx.fill();
      ctx.strokeStyle = rgba(RGB_CRIMSON, 0.78);
      ctx.lineWidth = 2;
      ctx.stroke();

      // Expanding inner detonation ring
      ctx.beginPath();
      ctx.arc(h.x, h.y, Math.max(2, h.radius * p), 0, TAU);
      ctx.strokeStyle = rgba(RGB_CRIMSON, 0.95);
      ctx.lineWidth = 2.5;
      ctx.stroke();

      // Crosshair tick
      ctx.beginPath();
      ctx.moveTo(h.x - 10, h.y);
      ctx.lineTo(h.x + 10, h.y);
      ctx.moveTo(h.x, h.y - 10);
      ctx.lineTo(h.x, h.y + 10);
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.75)';
      ctx.lineWidth = 1.5;
      ctx.stroke();
    }

    // Rings (frustum culled)
    for (let i = 0; i < this.ringActive; i++) {
      const r = this.rings[i];
      if (
        r.x + r.radius < viewMinX ||
        r.x - r.radius > viewMaxX ||
        r.y + r.radius < viewMinY ||
        r.y - r.radius > viewMaxY
      ) {
        continue;
      }
      ctx.beginPath();
      ctx.arc(r.x, r.y, Math.max(0.5, r.radius), 0, TAU);
      ctx.strokeStyle = rgba(r.rgb, Math.max(0, Math.min(1, r.life / r.maxLife)));
      ctx.lineWidth = 3;
      ctx.stroke();
    }

    // Batched Gems (2 draw calls total instead of hundreds)
    ctx.fillStyle = rgba(RGB_CYAN, 1);
    ctx.beginPath();
    for (let i = 0; i < this.gemActive; i++) {
      const g = this.gems[i];
      if (g.shard > 0) continue;
      if (g.x < viewMinX || g.x > viewMaxX || g.y < viewMinY || g.y > viewMaxY) continue;
      ctx.moveTo(g.x + 3.5, g.y);
      ctx.arc(g.x, g.y, 3.5, 0, TAU);
    }
    ctx.fill();

    ctx.fillStyle = rgba(RGB_GOLD, 1);
    ctx.beginPath();
    for (let i = 0; i < this.gemActive; i++) {
      const g = this.gems[i];
      if (g.shard <= 0) continue;
      if (g.x < viewMinX || g.x > viewMaxX || g.y < viewMinY || g.y > viewMaxY) continue;
      ctx.moveTo(g.x + 5.5, g.y);
      ctx.arc(g.x, g.y, 5.5, 0, TAU);
    }
    ctx.fill();

    // Batched Bullets (2 stroke calls total)
    if (this.bulletActive > 0) {
      ctx.lineCap = 'round';
      ctx.beginPath();
      for (let i = 0; i < this.bulletActive; i++) {
        const b = this.bullets[i];
        if (b.x < viewMinX || b.x > viewMaxX || b.y < viewMinY || b.y > viewMaxY) continue;
        ctx.moveTo(b.x, b.y);
        ctx.lineTo(b.x - b.vx * 0.028, b.y - b.vy * 0.028);
      }
      ctx.strokeStyle = 'rgba(5, 217, 232, 0.34)';
      ctx.lineWidth = 7;
      ctx.stroke();
      ctx.strokeStyle = '#ffffff';
      ctx.lineWidth = 2.5;
      ctx.stroke();
    }

    // Batched Tesla arcs
    if (this.arcActive > 0) {
      ctx.beginPath();
      for (let i = 0; i < this.arcActive; i++) {
        const a = this.arcs[i];
        const mx = (a.ax + a.bx) * 0.5 + randRange(-12, 12);
        const my = (a.ay + a.by) * 0.5 + randRange(-12, 12);
        ctx.moveTo(a.ax, a.ay);
        ctx.lineTo(mx, my);
        ctx.lineTo(a.bx, a.by);
      }
      ctx.strokeStyle = rgba(RGB_LIGHT_CYAN, 1);
      ctx.lineWidth = 2.5;
      ctx.stroke();
    }

    // Batched Enemy Bullets (3 fast passes: glow, crimson body, white core)
    if (this.eBulletActive > 0) {
      ctx.fillStyle = 'rgba(255, 58, 92, 0.30)';
      ctx.beginPath();
      for (let i = 0; i < this.eBulletActive; i++) {
        const eb = this.eBullets[i];
        if (eb.x < viewMinX || eb.x > viewMaxX || eb.y < viewMinY || eb.y > viewMaxY) continue;
        const gr = eb.radius * 1.9;
        ctx.moveTo(eb.x + gr, eb.y);
        ctx.arc(eb.x, eb.y, gr, 0, TAU);
      }
      ctx.fill();

      ctx.fillStyle = rgba(RGB_CRIMSON, 1);
      ctx.beginPath();
      for (let i = 0; i < this.eBulletActive; i++) {
        const eb = this.eBullets[i];
        if (eb.x < viewMinX || eb.x > viewMaxX || eb.y < viewMinY || eb.y > viewMaxY) continue;
        ctx.moveTo(eb.x + eb.radius, eb.y);
        ctx.arc(eb.x, eb.y, eb.radius, 0, TAU);
      }
      ctx.fill();

      ctx.fillStyle = '#ffffff';
      ctx.beginPath();
      for (let i = 0; i < this.eBulletActive; i++) {
        const eb = this.eBullets[i];
        if (eb.x < viewMinX || eb.x > viewMaxX || eb.y < viewMinY || eb.y > viewMaxY) continue;
        const cr = eb.radius * 0.45;
        ctx.moveTo(eb.x + cr, eb.y);
        ctx.arc(eb.x, eb.y, cr, 0, TAU);
      }
      ctx.fill();
    }

    // Enemies (frustum culled, 7 archetypes + elite halo + sniper laser + vortex field)
    for (let i = 0; i < this.enemyActive; i++) {
      const e = this.enemies[i];
      if (!e.alive) continue;
      if (
        e.x + e.radius + 20 < viewMinX ||
        e.x - e.radius - 20 > viewMaxX ||
        e.y + e.radius + 20 < viewMinY ||
        e.y - e.radius - 20 > viewMaxY
      ) {
        continue;
      }

      // Sniper laser-sight telegraph before firing
      if (e.kind === 4 && e.shootCd > 0 && e.shootCd < 0.45) {
        ctx.beginPath();
        ctx.moveTo(e.x, e.y);
        ctx.lineTo(this.playerX, this.playerY);
        ctx.strokeStyle = `rgba(255, 58, 92, ${0.35 + (0.45 - e.shootCd) * 1.4})`;
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }

      // Vortex gravity field visual
      if (e.kind === 6) {
        ctx.beginPath();
        ctx.arc(e.x, e.y, 44 + Math.sin(e.timer * 5) * 6, 0, TAU);
        ctx.strokeStyle = 'rgba(125, 110, 255, 0.28)';
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }

      const enragedBoss = e.kind === 3 && e.hp < e.maxHp * 0.5;
      const baseRgb =
        e.flash > 0
          ? RGB_WHITE
          : e.kind === 1
            ? RGB_ORANGE
            : e.kind === 2
              ? RGB_GOLD
              : e.kind === 3
                ? enragedBoss
                  ? RGB_CRIMSON
                  : RGB_PURPLE
                : e.kind === 4
                  ? RGB_CRIMSON
                  : e.kind === 5
                    ? RGB_LIME
                    : e.kind === 6
                      ? RGB_INDIGO
                      : RGB_PINK;
      const sides =
        e.kind === 0
          ? 3
          : e.kind === 1 || e.kind === 5
            ? 4
            : e.kind === 4
              ? 5
              : e.kind === 2
                ? 6
                : e.kind === 6
                  ? 7
                  : 8;

      if (e.elite) {
        ctx.beginPath();
        ctx.arc(e.x, e.y, e.radius + 5, 0, TAU);
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.65)';
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }

      this.drawPolyLines(e.x, e.y, sides, e.radius, e.angle, 7, rgba(baseRgb, 0.28));
      this.drawPolyLines(e.x, e.y, sides, e.radius, e.angle, 2.2, rgba(baseRgb, 1));

      if ((e.kind === 2 || e.kind === 3 || e.kind === 6 || e.elite) && e.hp < e.maxHp) {
        const bw = e.radius * 2;
        const ratio = Math.max(0, Math.min(1, e.hp / e.maxHp));
        ctx.fillStyle = 'rgba(255, 255, 255, 0.2)';
        ctx.fillRect(e.x - e.radius, e.y - e.radius - 10, bw, 4);
        ctx.fillStyle = rgba(baseRgb, 1);
        ctx.fillRect(e.x - e.radius, e.y - e.radius - 10, bw * ratio, 4);
      }
    }

    // Orbital saws
    if (this.orbitCount > 0) {
      ctx.strokeStyle = rgba(RGB_GREEN, 1);
      ctx.lineWidth = 2.5;
      ctx.beginPath();
      for (let i = 0; i < this.orbitCount; i++) {
        const a = this.orbitAngle + (i * TAU) / this.orbitCount;
        const bx = this.playerX + Math.cos(a) * 74;
        const by = this.playerY + Math.sin(a) * 74;
        ctx.moveTo(bx + 8.5, by);
        ctx.arc(bx, by, 8.5, 0, TAU);
      }
      ctx.stroke();
    }

    // Phantom Wraith Plasma Mines
    for (let i = 0; i < this.mineActive; i++) {
      const m = this.mines[i];
      if (m.x + 28 < viewMinX || m.x - 28 > viewMaxX || m.y + 28 < viewMinY || m.y - 28 > viewMaxY) {
        continue;
      }
      ctx.beginPath();
      ctx.arc(m.x, m.y, 18 + Math.sin(m.life * 8) * 3, 0, TAU);
      ctx.strokeStyle = 'rgba(184, 41, 234, 0.42)';
      ctx.lineWidth = 1.5;
      ctx.stroke();
      this.drawPolyLines(m.x, m.y, 6, 9, m.life * 2.5, 2.2, rgba(RGB_PURPLE, 1));
      ctx.beginPath();
      ctx.arc(m.x, m.y, 3.5, 0, TAU);
      ctx.fillStyle = rgba(RGB_PINK, 1);
      ctx.fill();
    }

    // Player ship (4 distinct hulls + plasma shield aura + Overdrive / Dash glow)
    const px = this.playerX;
    const py = this.playerY;
    const shipId = this.meta.selectedShip;
    const shipRgb = this.getSelectedShip().rgb;
    const ca = Math.cos(this.aimAngle);
    const sa = Math.sin(this.aimAngle);
    const rx = -sa;
    const ry = ca;

    if (this.maxShield > 0 && this.shield > 0.5) {
      const sAlpha = 0.28 + 0.42 * (this.shield / this.maxShield);
      ctx.beginPath();
      ctx.arc(px, py, 25, 0, TAU);
      ctx.strokeStyle = rgba(RGB_LIGHT_CYAN, sAlpha);
      ctx.lineWidth = 2.2;
      ctx.stroke();
    }

    ctx.beginPath();
    if (shipId === 1) {
      // VIPER MK-II: razor-sharp needle interceptor with swept forward wings
      ctx.moveTo(px + ca * 24, py + sa * 24);
      ctx.lineTo(px - ca * 4 + rx * 16, py - sa * 4 + ry * 16);
      ctx.lineTo(px - ca * 15 + rx * 7, py - sa * 15 + ry * 7);
      ctx.lineTo(px - ca * 8, py - sa * 8);
      ctx.lineTo(px - ca * 15 - rx * 7, py - sa * 15 - ry * 7);
      ctx.lineTo(px - ca * 4 - rx * 16, py - sa * 4 - ry * 16);
      ctx.closePath();
    } else if (shipId === 2) {
      // TITAN DREADNOUGHT: wide heavy armored hexagonal gunship
      ctx.moveTo(px + ca * 19 + rx * 6, py + sa * 19 + ry * 6);
      ctx.lineTo(px + ca * 6 + rx * 18, py + sa * 6 + ry * 18);
      ctx.lineTo(px - ca * 15 + rx * 14, py - sa * 15 + ry * 14);
      ctx.lineTo(px - ca * 18, py - sa * 18);
      ctx.lineTo(px - ca * 15 - rx * 14, py - sa * 15 - ry * 14);
      ctx.lineTo(px + ca * 6 - rx * 18, py + sa * 6 - ry * 18);
      ctx.lineTo(px + ca * 19 - rx * 6, py + sa * 19 - ry * 6);
      ctx.closePath();
    } else if (shipId === 3) {
      // PHANTOM WRAITH: stealth twin-fin crescent/diamond
      ctx.moveTo(px + ca * 22, py + sa * 22);
      ctx.lineTo(px + ca * 4 + rx * 13, py + sa * 4 + ry * 13);
      ctx.lineTo(px - ca * 17 + rx * 15, py - sa * 17 + ry * 15);
      ctx.lineTo(px - ca * 6, py - sa * 6);
      ctx.lineTo(px - ca * 17 - rx * 15, py - sa * 17 - ry * 15);
      ctx.lineTo(px + ca * 4 - rx * 13, py + sa * 4 - ry * 13);
      ctx.closePath();
    } else {
      // INTERCEPTOR: classic delta fighter
      const cl = Math.cos(this.aimAngle + 2.4);
      const sl = Math.sin(this.aimAngle + 2.4);
      const cr = Math.cos(this.aimAngle - 2.4);
      const sr = Math.sin(this.aimAngle - 2.4);
      ctx.moveTo(px + ca * 20, py + sa * 20);
      ctx.lineTo(px + cl * 15, py + sl * 15);
      ctx.lineTo(px - ca * 7, py - sa * 7);
      ctx.lineTo(px + cr * 15, py + sr * 15);
      ctx.closePath();
    }

    ctx.strokeStyle =
      this.overdriveTimer > 0 ? 'rgba(249, 200, 14, 0.45)' : rgba(shipRgb, 0.34);
    ctx.lineWidth = 7.5;
    ctx.stroke();
    ctx.strokeStyle =
      this.dashTimer > 0
        ? '#ffffff'
        : this.overdriveTimer > 0
          ? rgba(RGB_GOLD, 1)
          : rgba(shipRgb, 1);
    ctx.lineWidth = 2.5;
    ctx.stroke();

    // Off-screen Overload Beacon directional gold diamond around ship
    if (this.beacon.active) {
      const bdx = this.beacon.x - px;
      const bdy = this.beacon.y - py;
      if (Math.abs(bdx) > sw * 0.44 || Math.abs(bdy) > sh * 0.44) {
        const bd = Math.sqrt(bdx * bdx + bdy * bdy) || 1;
        const ix = px + (bdx / bd) * 56;
        const iy = py + (bdy / bd) * 56;
        this.drawPolyLines(ix, iy, 4, 6.5, Math.atan2(bdy, bdx), 2.2, rgba(RGB_GOLD, 1));
      }
    }

    // Off-screen enemy radar dots around ship
    for (let idx = 0; idx < this.enemyActive; idx++) {
      const e = this.enemies[idx];
      if (!e.alive) continue;
      if (e.kind !== 3 && !e.elite && (idx & 3) !== 0) continue;
      const dx = e.x - px;
      const dy = e.y - py;
      if (Math.abs(dx) > sw * 0.48 || Math.abs(dy) > sh * 0.48) {
        const d = Math.sqrt(dx * dx + dy * dy) || 1;
        ctx.beginPath();
        ctx.arc(px + (dx / d) * 46, py + (dy / d) * 46, e.kind === 3 ? 4.5 : e.elite ? 3.5 : 2.5, 0, TAU);
        ctx.fillStyle =
          e.kind === 3
            ? rgba(RGB_PURPLE, 1)
            : e.elite
              ? '#ffffff'
              : 'rgba(255, 42, 109, 0.67)';
        ctx.fill();
      }
    }

    // Particles (frustum culled)
    for (let i = 0; i < this.particleActive; i++) {
      const pt = this.particles[i];
      if (pt.x < viewMinX || pt.x > viewMaxX || pt.y < viewMinY || pt.y > viewMaxY) continue;
      ctx.fillStyle = rgba(pt.rgb, Math.max(0, Math.min(1, pt.life / pt.maxLife)));
      ctx.fillRect(pt.x - pt.size * 0.5, pt.y - pt.size * 0.5, pt.size, pt.size);
    }

    // Damage numbers (frustum culled, single cached font)
    if (this.textActive > 0) {
      this.setFontSize(14);
      ctx.textAlign = 'left';
      ctx.textBaseline = 'alphabetic';
      for (let i = 0; i < this.textActive; i++) {
        const t = this.texts[i];
        if (t.x < viewMinX || t.x > viewMaxX || t.y < viewMinY || t.y > viewMaxY) continue;
        ctx.fillStyle = rgba(t.rgb, Math.max(0, Math.min(1, t.life * 2)));
        ctx.fillText(t.val, t.x - 10, t.y);
      }
    }

    ctx.restore();
  }

  private renderHud(): void {
    const ctx = this.ctx;
    const sw = this.width;
    const sh = this.height;
    const compact = sw < 520;
    const pauseRect = this.getPauseRect();

    // Left block: HULL, optional Plasma SHIELD, & XP bars (responsive width so it never crowds WAVE/SHARDS)
    const barW = Math.max(88, Math.min(168, sw * (compact ? 0.25 : 0.28)));
    const lblSize = compact ? 12 : 14;
    this.drawText(`HULL ${Math.ceil(this.hp)}/${Math.round(this.maxHp)}`, 12, 22, lblSize, '#ffffff');
    ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.fillRect(12, 28, barW, 8);
    ctx.fillStyle = rgba(RGB_PINK, 1);
    ctx.fillRect(12, 28, barW * Math.max(0, Math.min(1, this.hp / this.maxHp)), 8);
    ctx.strokeStyle = rgba(RGB_CYAN, 1);
    ctx.lineWidth = 1;
    ctx.strokeRect(12, 28, barW, 8);

    let leftY = 40;
    if (this.maxShield > 0) {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.08)';
      ctx.fillRect(12, leftY, barW, 6);
      ctx.fillStyle = rgba(RGB_LIGHT_CYAN, 1);
      ctx.fillRect(12, leftY, barW * Math.max(0, Math.min(1, this.shield / this.maxShield)), 6);
      ctx.strokeStyle = rgba(RGB_LIGHT_CYAN, 0.75);
      ctx.lineWidth = 1;
      ctx.strokeRect(12, leftY, barW, 6);
      leftY += 10;
    }

    const lvlLabelY = leftY + 12;
    const lvlBarY = leftY + 18;
    this.drawText(`LVL ${this.level}`, 12, lvlLabelY, lblSize, '#ffffff');
    ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.fillRect(12, lvlBarY, barW, 8);
    ctx.fillStyle = rgba(RGB_CYAN, 1);
    ctx.fillRect(12, lvlBarY, barW * Math.max(0, Math.min(1, this.xp / this.nextXp)), 8);
    ctx.strokeStyle = rgba(RGB_CYAN, 1);
    ctx.strokeRect(12, lvlBarY, barW, 8);

    // Subtle red edge damage flash
    if (this.hurtFlash > 0.01) {
      ctx.strokeStyle = rgba(RGB_CRIMSON, this.hurtFlash * 0.55);
      ctx.lineWidth = 18;
      ctx.strokeRect(0, 0, sw, sh);
    }

    // Right block: SHARDS, FPS/DIFF, COMBO & OVERDRIVE right-aligned strictly left of Pause button
    const rightEdge = pauseRect.x - 10;
    const shardText = `SHARDS: ${this.runShards}`;
    const shardSize = compact ? 12 : 14;
    const diff = this.getDifficulty();
    this.drawRightText(shardText, rightEdge, 22, shardSize, rgba(RGB_GOLD, 1));
    const metaRight = diff.id > 0 ? `${diff.name} • ${this.fps} FPS` : `${this.fps} FPS`;
    this.drawRightText(
      metaRight,
      rightEdge,
      40,
      compact ? 10 : 12,
      diff.id > 0 ? rgba(diff.rgb, 1) : '#94a3b8'
    );

    let rightRowY = 56;
    if (this.combo >= 3) {
      const mult = (1 + Math.min(1.0, this.combo * 0.025)).toFixed(2);
      this.drawRightText(`COMBO ${this.combo} (${mult}x)`, rightEdge, rightRowY, compact ? 10 : 12, rgba(RGB_CYAN, 1));
      rightRowY += 16;
    }
    if (this.overdriveTimer > 0) {
      this.drawRightText(`OVERDRIVE ${this.overdriveTimer.toFixed(1)}s`, rightEdge, rightRowY, compact ? 10 : 12, rgba(RGB_GOLD, 1));
    }

    // Center block: WAVE + timer dynamically centered in the free gap between Left and Right blocks
    this.setFontSize(shardSize);
    const shardW = ctx.measureText(shardText).width;
    const leftEnd = 12 + barW + 10;
    const rightStart = rightEdge - shardW - 10;
    const freeGap = rightStart - leftEnd;
    const waveLabel = `WAVE ${this.wave}`;
    const eventTag = this.waveEvent === 'NORMAL' ? '' : ` [${this.waveEvent}]`;
    const timerLabel = `${Math.max(1, Math.ceil(22 - this.waveTimer))}s${eventTag}`;

    if (freeGap >= 92) {
      const waveCx = leftEnd + freeGap * 0.5;
      this.drawCenteredText(waveLabel, waveCx, 24, compact ? 15 : 19, rgba(RGB_PINK, 1));
      this.drawCenteredText(timerLabel, waveCx, 42, compact ? 10 : 12, '#cbd5e1');
    } else {
      // Ultra-narrow viewport fallback: place WAVE cleanly below the top row
      this.drawCenteredText(`${waveLabel} • ${timerLabel}`, sw * 0.5, lvlBarY + 26, 13, rgba(RGB_PINK, 1));
    }

    // Wave Event / Beacon announcement banner (cleanly below top HUD, zero overlap)
    if (this.bannerTimer > 0) {
      const alpha = Math.min(1, this.bannerTimer / 0.75);
      const by = freeGap >= 92 ? 64 : lvlBarY + 42;
      const bw = Math.min(sw - 170, 390);
      if (bw > 150) {
        const bx = sw * 0.5 - bw * 0.5;
        ctx.fillStyle = `rgba(6, 12, 24, ${0.76 * alpha})`;
        ctx.fillRect(bx, by, bw, 40);
        ctx.strokeStyle = rgba(this.bannerRgb, 0.85 * alpha);
        ctx.lineWidth = 1.5;
        ctx.strokeRect(bx, by, bw, 40);
        this.drawCenteredText(this.bannerText, sw * 0.5, by + 17, compact ? 11 : 13, rgba(this.bannerRgb, alpha));
        this.drawCenteredText(this.bannerSub, sw * 0.5, by + 32, compact ? 9 : 10, `rgba(226, 232, 240, ${alpha})`);
      }
    }

    // Pause button
    ctx.fillStyle = 'rgba(10, 16, 30, 0.86)';
    ctx.fillRect(pauseRect.x, pauseRect.y, pauseRect.w, pauseRect.h);
    ctx.strokeStyle = rgba(RGB_CYAN, 1);
    ctx.lineWidth = 1.5;
    ctx.strokeRect(pauseRect.x, pauseRect.y, pauseRect.w, pauseRect.h);
    this.drawCenteredText(
      '||',
      pauseRect.x + pauseRect.w * 0.5,
      pauseRect.y + pauseRect.h * 0.65,
      compact ? 14 : 17,
      rgba(RGB_CYAN, 1)
    );

    // Compact tactical minimap (top-left below XP bar) for the enlarged 2800-radius arena
    const mapR = compact ? 26 : 32;
    const mapCx = 12 + mapR;
    const mapCy = lvlBarY + 20 + mapR;
    ctx.beginPath();
    ctx.arc(mapCx, mapCy, mapR, 0, TAU);
    ctx.fillStyle = 'rgba(6, 12, 24, 0.72)';
    ctx.fill();
    ctx.strokeStyle = 'rgba(255, 42, 109, 0.65)';
    ctx.lineWidth = 1.5;
    ctx.stroke();

    const mapScale = (mapR - 3) / ARENA_RADIUS;
    // Overload Beacon blip on minimap (bright gold square)
    if (this.beacon.active) {
      const bmx = mapCx + this.beacon.x * mapScale;
      const bmy = mapCy + this.beacon.y * mapScale;
      ctx.fillStyle = rgba(RGB_GOLD, 1);
      ctx.fillRect(bmx - 2.5, bmy - 2.5, 5, 5);
    }
    // Enemy blips on minimap
    for (let i = 0; i < this.enemyActive; i++) {
      const e = this.enemies[i];
      if (!e.alive) continue;
      const mx = mapCx + e.x * mapScale;
      const my = mapCy + e.y * mapScale;
      ctx.fillStyle =
        e.kind === 3
          ? rgba(RGB_PURPLE, 1)
          : e.elite
            ? '#ffffff'
            : 'rgba(255, 42, 109, 0.75)';
      const dot = e.kind === 3 ? 3 : e.elite ? 2.2 : 1.5;
      ctx.fillRect(mx - dot * 0.5, my - dot * 0.5, dot, dot);
    }
    // Player blip on minimap
    ctx.beginPath();
    ctx.arc(mapCx + this.playerX * mapScale, mapCy + this.playerY * mapScale, 2.5, 0, TAU);
    ctx.fillStyle = this.overdriveTimer > 0 ? rgba(RGB_GOLD, 1) : rgba(RGB_CYAN, 1);
    ctx.fill();

    // Dash button
    const dcx = sw - 58;
    const dcy = sh - 62;
    const dashCol = this.dashCd > 0 ? 'rgba(70, 90, 120, 0.75)' : rgba(RGB_CYAN, 1);
    ctx.beginPath();
    ctx.arc(dcx, dcy, 35, 0, TAU);
    ctx.fillStyle = 'rgba(5, 217, 232, 0.1)';
    ctx.fill();
    ctx.strokeStyle = dashCol;
    ctx.lineWidth = 2;
    ctx.stroke();
    this.drawCenteredText('DASH', dcx, dcy - 2, 14, dashCol);
    this.drawCenteredText(this.dashCd > 0 ? `${this.dashCd.toFixed(1)}s` : 'READY', dcx, dcy + 15, 11, dashCol);

    // Joysticks
    if (this.leftStick.touchId === null) {
      ctx.beginPath();
      ctx.arc(86, sh - 96, 42, 0, TAU);
      ctx.strokeStyle = 'rgba(5, 217, 232, 0.25)';
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(86, sh - 96, 12, 0, TAU);
      ctx.fillStyle = 'rgba(5, 217, 232, 0.18)';
      ctx.fill();
    } else {
      ctx.beginPath();
      ctx.arc(this.leftStick.startX, this.leftStick.startY, 44, 0, TAU);
      ctx.strokeStyle = rgba(RGB_CYAN, 1);
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(this.leftStick.curX, this.leftStick.curY, 16, 0, TAU);
      ctx.fillStyle = rgba(RGB_CYAN, 1);
      ctx.fill();
    }

    if (this.rightStick.touchId !== null) {
      ctx.beginPath();
      ctx.arc(this.rightStick.startX, this.rightStick.startY, 44, 0, TAU);
      ctx.strokeStyle = rgba(RGB_PINK, 1);
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(this.rightStick.curX, this.rightStick.curY, 16, 0, TAU);
      ctx.fillStyle = rgba(RGB_PINK, 1);
      ctx.fill();
    }
  }

  private renderMenu(): void {
    const sw = this.width;
    const sh = this.height;
    const compact = sw < 600 || sh < 520;

    const prestigeTag = this.meta.prestigeLvl > 0 ? `  •  PRESTIZ ★${this.meta.prestigeLvl}` : '';
    this.drawCenteredText(
      `NEON CORE • HANGAR & SKLEP${prestigeTag}`,
      sw * 0.5,
      compact ? 22 : 28,
      compact ? 10 : 12,
      rgba(RGB_PINK, 1)
    );
    this.drawCenteredText('NEON CORE', sw * 0.5, compact ? 50 : 62, compact ? 26 : 34, rgba(RGB_CYAN, 1));

    const recStr =
      this.meta.bestWave > 0
        ? `  •  REKORD: FALA ${this.meta.bestWave} (${this.meta.bestKills} KILLS)`
        : '';
    this.drawCenteredText(
      `SHARDS: ${this.meta.shards}${recStr}`,
      sw * 0.5,
      compact ? 70 : 84,
      compact ? 12 : 14,
      rgba(RGB_GOLD, 1)
    );

    // 4 Hangar tabs: BOJOWE | SURVIVAL | ENDGAME | STATKI
    const tabY = compact ? 80 : 96;
    const tabGap = 6;
    const totalTabW = Math.min(580, sw - 24);
    const tabW = (totalTabW - tabGap * 3) / 4;
    const tabH = compact ? 28 : 32;
    const tabStartX = sw * 0.5 - totalTabW * 0.5;
    const tabDefs: Array<{ idx: 0 | 1 | 2 | 3; label: string; rgb: [number, number, number] }> = [
      { idx: 0, label: compact ? 'BOJOWE' : 'BOJOWE (5)', rgb: RGB_CYAN },
      { idx: 1, label: compact ? 'SURVIVAL' : 'SURVIVAL (5)', rgb: RGB_GREEN },
      { idx: 2, label: compact ? 'ENDGAME' : 'ENDGAME (5)', rgb: RGB_GOLD },
      { idx: 3, label: compact ? 'STATKI' : 'STATKI (4)', rgb: RGB_PURPLE },
    ];
    for (let t = 0; t < tabDefs.length; t++) {
      const td = tabDefs[t];
      const active = this.shopTab === td.idx;
      if (
        this.uiButton(
          { x: tabStartX + t * (tabW + tabGap), y: tabY, w: tabW, h: tabH },
          td.label,
          null,
          active ? td.rgb : RGB_SLATE,
          active
        )
      ) {
        this.shopTab = td.idx;
      }
    }

    const cols = sw >= 580 ? 2 : 1;
    const startY = tabY + tabH + (compact ? 8 : 11);
    let cardsEndY = startY;

    if (this.shopTab === 3) {
      // Unlockable Ship Classes (4)
      const cardW = cols === 2 ? Math.min(285, (sw - 48) * 0.5) : Math.min(360, sw - 24);
      const cardH = compact ? 48 : 62;
      const gapY = compact ? 6 : 9;
      const startX = sw * 0.5 - (cols * cardW + (cols - 1) * 10) * 0.5;

      for (let i = 0; i < SHIPS.length; i++) {
        const ship = SHIPS[i];
        const r = Math.floor(i / cols);
        const c = i % cols;
        const rx = startX + c * (cardW + 10);
        const ry = startY + r * (cardH + gapY);

        const unlocked = (this.meta.unlockedShipsMask & (1 << ship.id)) !== 0;
        const selected = this.meta.selectedShip === ship.id;
        const canBuy = !unlocked && this.meta.shards >= ship.cost;
        const border = selected ? ship.rgb : unlocked ? RGB_CYAN : canBuy ? RGB_GOLD : RGB_SLATE;
        const status = selected ? '[WYBRANY]' : unlocked ? '[WYBIERZ]' : `[${ship.cost} SHARDS]`;
        const title = `${ship.name} ${status}`;
        const sub = ship.perk;

        if (this.uiButton({ x: rx, y: ry, w: cardW, h: cardH }, title, sub, border, selected || canBuy)) {
          if (unlocked) {
            this.meta.selectedShip = ship.id;
            saveMeta(this.meta);
            this.sounds.play('hit', this.settings.sound, 0.55);
          } else if (canBuy) {
            this.meta.shards -= ship.cost;
            this.meta.unlockedShipsMask |= 1 << ship.id;
            this.meta.selectedShip = ship.id;
            saveMeta(this.meta);
            this.sounds.play('lvlup', this.settings.sound, 0.9);
          }
        }
      }
      const rows = Math.ceil(SHIPS.length / cols);
      cardsEndY = startY + rows * (cardH + gapY);
    } else {
      // Permanent Upgrade Shop (BOJOWE / SURVIVAL / ENDGAME)
      const items = SHOP_ITEMS.filter((it) => it.tab === this.shopTab);
      const cardW = cols === 2 ? Math.min(285, (sw - 48) * 0.5) : Math.min(360, sw - 24);
      const cardH = compact ? 40 : 48;
      const gapY = compact ? 5 : 8;
      const startX = sw * 0.5 - (cols * cardW + (cols - 1) * 10) * 0.5;

      for (let idx = 0; idx < items.length; idx++) {
        const item = items[idx];
        const r = Math.floor(idx / cols);
        const c = idx % cols;
        const isLastOdd = cols === 2 && idx === items.length - 1 && items.length % 2 === 1;
        const rx = isLastOdd ? sw * 0.5 - cardW * 0.5 : startX + c * (cardW + 10);
        const ry = startY + r * (cardH + gapY);

        const lvl = this.meta[item.key];
        const maxed = lvl >= item.maxLvl;
        const cost = item.quadratic ? (lvl + 1) * (lvl + 1) * item.baseCost : (lvl + 1) * item.baseCost;
        const canBuy = !maxed && this.meta.shards >= cost;
        const border = maxed ? RGB_GREEN : canBuy ? RGB_GOLD : RGB_SLATE;
        const title = maxed ? `${item.name} [MAX]` : `${item.name} [${lvl}/${item.maxLvl}]`;
        const sub = maxed ? `${item.bonus} • MAKSIMUM` : `${item.bonus} • ${cost} SHARDS`;

        if (this.uiButton({ x: rx, y: ry, w: cardW, h: cardH }, title, sub, border, canBuy)) {
          if (canBuy) {
            this.meta.shards -= cost;
            this.meta[item.key] = lvl + 1;
            saveMeta(this.meta);
            this.sounds.play('lvlup', this.settings.sound, 0.8);
          }
        }
      }
      const rows = Math.ceil(items.length / cols);
      cardsEndY = startY + rows * (cardH + gapY);
    }

    // Difficulty selector row: NORMAL 1x | HARDCORE 1.6x | NIGHTMARE 2.5x
    const diffY = cardsEndY + (compact ? 4 : 8);
    const diffGap = 6;
    const diffTotalW = Math.min(480, sw - 24);
    const diffW = (diffTotalW - diffGap * 2) / 3;
    const diffH = compact ? 28 : 34;
    const diffStartX = sw * 0.5 - diffTotalW * 0.5;
    for (let d = 0; d < DIFFICULTIES.length; d++) {
      const df = DIFFICULTIES[d];
      const active = this.meta.difficulty === df.id;
      if (
        this.uiButton(
          { x: diffStartX + d * (diffW + diffGap), y: diffY, w: diffW, h: diffH },
          df.shortTag,
          null,
          active ? df.rgb : RGB_SLATE,
          active
        )
      ) {
        this.meta.difficulty = df.id;
        saveMeta(this.meta);
      }
    }

    // Bottom action row: START GRY | OPCJE
    const btnY = diffY + diffH + (compact ? 7 : 10);
    const btnW = Math.min(185, (sw - 34) * 0.5);
    const btnH = compact ? 38 : 44;
    const activeShip = this.getSelectedShip();

    if (
      this.uiButton(
        { x: sw * 0.5 - btnW - 6, y: btnY, w: btnW, h: btnH },
        `START: ${activeShip.name.split(' ')[0]}`,
        null,
        activeShip.rgb,
        true
      )
    ) {
      this.startRun();
    }
    if (this.uiButton({ x: sw * 0.5 + 6, y: btnY, w: btnW, h: btnH }, 'OPCJE', null, RGB_GOLD)) {
      this.prevState = { kind: 'Menu' };
      this.state = { kind: 'Settings' };
    }

    if (btnY + btnH + 22 < sh) {
      this.drawCenteredText(
        'WASD/STRZALKI — RUCH  •  SPACJA — DASH  •  ESC — PAUZA',
        sw * 0.5,
        btnY + btnH + 18,
        10,
        '#64748b'
      );
    }
  }

  private tick(dt: number): void {
    const ctx = this.ctx;
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.fillStyle = '#04060b';
    ctx.fillRect(0, 0, this.width, this.height);

    const sw = this.width;
    const sh = this.height;

    if (this.state.kind === 'Menu') {
      this.renderMenu();
    } else if (this.state.kind === 'Playing') {
      this.updatePlaying(dt);
      this.renderWorld();
      this.renderHud();
    } else if (this.state.kind === 'Paused') {
      this.renderWorld();
      ctx.fillStyle = 'rgba(4, 6, 11, 0.82)';
      ctx.fillRect(0, 0, sw, sh);
      this.drawCenteredText('PAUZA', sw * 0.5, sh * 0.28, 34, rgba(RGB_CYAN, 1));
      if (this.uiButton({ x: sw * 0.5 - 120, y: sh * 0.38, w: 240, h: 48 }, 'WZNOW', null, RGB_CYAN)) {
        this.state = { kind: 'Playing' };
      }
      if (this.uiButton({ x: sw * 0.5 - 120, y: sh * 0.38 + 60, w: 240, h: 48 }, 'OPCJE', null, RGB_GOLD)) {
        this.prevState = { kind: 'Paused' };
        this.state = { kind: 'Settings' };
      }
      if (this.uiButton({ x: sw * 0.5 - 120, y: sh * 0.38 + 120, w: 240, h: 48 }, 'PRZERWIJ LOT', null, RGB_PINK)) {
        this.finishRun(false);
      }
    } else if (this.state.kind === 'Settings') {
      this.drawCenteredText('OPCJE SYSTEMU', sw * 0.5, 65, 28, rgba(RGB_CYAN, 1));
      const bw = Math.min(320, sw - 40);
      const bx = sw * 0.5 - bw * 0.5;

      if (this.uiButton({ x: bx, y: 105, w: bw, h: 44 }, this.settings.sound ? 'DZWIEK: WLACZONY' : 'DZWIEK: WYLACZONY', null, RGB_CYAN)) {
        this.settings.sound = !this.settings.sound;
      }
      if (this.uiButton({ x: bx, y: 158, w: bw, h: 44 }, this.settings.shake ? 'SHAKE: WLACZONY' : 'SHAKE: WYLACZONY', null, RGB_CYAN)) {
        this.settings.shake = !this.settings.shake;
      }
      if (this.uiButton({ x: bx, y: 211, w: bw, h: 44 }, this.settings.dmgNums ? 'OBRAZENIA: WLACZONE' : 'OBRAZENIA: WYLACZONE', null, RGB_CYAN)) {
        this.settings.dmgNums = !this.settings.dmgNums;
      }
      if (this.uiButton({ x: bx, y: 264, w: bw, h: 44 }, this.settings.fixedStick ? 'DRAZEK: STALY' : 'DRAZEK: PLYWAJACY', null, RGB_CYAN)) {
        this.settings.fixedStick = !this.settings.fixedStick;
      }
      if (this.uiButton({ x: bx, y: 317, w: bw, h: 44 }, this.settings.autoAim ? 'AUTO-AIM: WLACZONY' : 'AUTO-AIM: WYLACZONY', null, RGB_CYAN)) {
        this.settings.autoAim = !this.settings.autoAim;
      }
      if (this.uiButton({ x: bx, y: 378, w: bw, h: 48 }, 'WROC', null, RGB_GOLD)) {
        this.state = this.prevState;
      }
    } else if (this.state.kind === 'LevelUp') {
      this.renderWorld();
      ctx.fillStyle = 'rgba(4, 6, 11, 0.84)';
      ctx.fillRect(0, 0, sw, sh);
      this.drawCenteredText('OVERCLOCK RDZENIA', sw * 0.5, 65, 26, rgba(RGB_CYAN, 1));
      const cw = Math.min(340, sw - 40);
      const choices = [...this.levelChoices];
      for (let i = 0; i < choices.length; i++) {
        const opt = choices[i];
        if (this.uiButton({ x: sw * 0.5 - cw * 0.5, y: 105 + i * 74, w: cw, h: 62 }, opt.title, opt.desc, RGB_CYAN)) {
          this.applyUpgrade(opt.id);
          break;
        }
      }
    } else if (this.state.kind === 'GameOver') {
      const title = this.state.victory ? 'SEKTOR DEMO OCZYSZCZONY' : 'RDZEN ZNISZCZONY';
      const diff = this.getDifficulty();
      const ship = this.getSelectedShip();
      this.drawCenteredText(title, sw * 0.5, sh * 0.26, 28, rgba(this.state.victory ? RGB_CYAN : RGB_PINK, 1));
      this.drawCenteredText(
        `${ship.name}  •  ${diff.name} (${diff.shardMult.toFixed(1)}x)`,
        sw * 0.5,
        sh * 0.26 + 32,
        14,
        rgba(diff.rgb, 1)
      );
      if (this.isNewRecord) {
        this.drawCenteredText('★ NOWY REKORD OSOBISTY! ★', sw * 0.5, sh * 0.26 + 58, 16, rgba(RGB_GOLD, 1));
      }
      const statY = sh * 0.26 + (this.isNewRecord ? 90 : 70);
      this.drawCenteredText(`FALA: ${this.wave}  |  POZIOM: ${this.level}`, sw * 0.5, statY, 17, '#ffffff');
      this.drawCenteredText(`ZNISZCZENI: ${this.kills}  |  SHARDS: +${this.runShards}`, sw * 0.5, statY + 30, 17, rgba(RGB_GOLD, 1));
      this.drawCenteredText(
        `NAJLEPSZA FALA: ${this.meta.bestWave}  |  REKORD KILLS: ${this.meta.bestKills}`,
        sw * 0.5,
        statY + 56,
        13,
        '#94a3b8'
      );
      if (this.uiButton({ x: sw * 0.5 - 135, y: statY + 80, w: 270, h: 50 }, 'POWROT DO HANGARU', null, RGB_CYAN, true)) {
        this.state = { kind: 'Menu' };
      }
    }
  }
}
