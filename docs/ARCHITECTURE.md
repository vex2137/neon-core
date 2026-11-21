# Architecture Overview

Neon Core is organized into focused, dependency-acyclic Rust modules built on Macroquad 0.4. Pure game simulation logic is decoupled from Macroquad windowing and input polling so tests and benchmarks run deterministically without a GPU context.

## Module Layout

```text
src/
├── main.rs                    # Entry point, Window Conf, Android JNI pause hook, main loop
├── rng.rs                     # Deterministic XorShift64* RNG (`GameRng`)
├── tests.rs                   # Integration, determinism, regression & combat benchmark tests
├── game/
│   ├── mod.rs                 # `Game` orchestrator and `step_playing(dt, FrameInput, Viewport)`
│   ├── config.rs              # Arena dimensions, entity capacity caps, `Viewport` virtual scaling
│   ├── state.rs               # `ScreenState`, `Settings`, zero-alloc `BannerState`
│   └── session.rs             # Run lifecycle (`start_run`, `pause_run`, `advance_wave`, `finish_run`)
├── player/
│   ├── mod.rs
│   ├── stats.rs               # `ShipSpec`, `SHIPS`, `PlayerState` stat computation & upgrades
│   ├── movement.rs            # Exponential acceleration, dash locomotion, thrust trails
│   ├── weapons.rs             # Aiming resolution (touch/auto-aim/mouse) & primary salvo firing
│   └── abilities.rs           # Dash trigger, cooldown enforcement, Phantom mine drop
├── enemies/
│   ├── mod.rs
│   ├── types.rs               # `EnemyKind` (7 archetypes), `Enemy` struct, polygon/color metadata
│   ├── spawning.rs            # Wave HP/speed scaling, elite rolls, spawn clamping
│   └── ai.rs                  # Per-archetype AI behaviors, sniper laser, vortex pull, boss phases
├── combat/
│   ├── mod.rs
│   ├── projectiles.rs         # `Bullet`, `EnemyBullet`, `Mine`
│   ├── damage.rs              # Pure crit roll, shield-before-hull damage, kill reward formulas
│   └── collisions.rs          # Enemy bullet, orbital hazard, and gem magnet/pickup loops
├── world/
│   ├── mod.rs
│   ├── spatial_grid.rs        # Fixed-capacity intrusive linked-list `SpatialGrid`
│   ├── arena.rs               # Circular arena clamping and parallax starfield generator
│   ├── hazards.rs             # Orbital Hazard Strike spawner and telegraphs
│   └── objectives.rs          # Overload Beacon capture objective
├── progression/
│   ├── mod.rs
│   ├── waves.rs               # `WaveEvent`, `DifficultySpec`, wave schedule & boss scaling
│   ├── experience.rs          # `Gem`, combo XP multiplier, next-level XP curve
│   ├── upgrades.rs            # `UpgradeCard`, `ALL_UPGRADES`, Fisher-Yates roll
│   └── shop.rs                # `ShopItem`, `SHOP_ITEMS`, overflow-safe cost formulas
├── effects/
│   ├── mod.rs                 # `EffectsPool` (particles, rings, arcs, texts, shake, hit-stop)
│   ├── floating_text.rs       # Zero-allocation stack `TextBuf<N>` and `DmgText`
│   └── particles.rs           # `Particle`, `Ring`, `ArcBolt`
├── audio/
│   ├── mod.rs                 # Target-gated `macroquad::audio` / WASM no-op shim
│   ├── procedural_audio.rs    # 16-bit 22.05 kHz PCM WAV synthesizer (`synth_wav`)
│   └── sound_bank.rs          # `SoundBank` async loader and throttled shoot playback
├── input/
│   ├── mod.rs
│   ├── touch.rs               # Multi-touch ID tracking, dynamic/fixed left stick, right aim stick
│   └── controls.rs            # Unified `FrameInput` polling keyboard, mouse, and touch
├── persistence/
│   ├── mod.rs
│   ├── save.rs                # `MetaSave`, clamping, atomic temp-file writes & `.bak` recovery
│   └── migration.rs           # Versioned v2 key=value + FNV-1a checksum & legacy v1 CSV parser
├── performance/
│   ├── mod.rs
│   └── metrics.rs             # Zero-allocation 120-frame rolling frame-time and entity telemetry
└── ui/
    ├── mod.rs                 # Shared palette, text alignment helpers, `ui_button_v`
    ├── world_render.rs        # Virtual-camera world renderer, frustum culling, stack ship verts
    ├── hud.rs                 # Zero-alloc HUD, minimap, dash cooldown arc, virtual sticks
    ├── menus.rs               # Hangar shop tabs, ship unlock/select, difficulty selector
    └── overlays.rs            # Paused, Settings, LevelUp, and GameOver modal screens
```

## Key Architectural Principles

1. **Unidirectional Data Flow**: `main.rs` polls platform input (`input::poll_playing_input`), passes an immutable `FrameInput` + `Viewport` into `Game::step_playing`, and hands `&Game` to `ui::world_render` and `ui::hud`.
2. **Explicit State Machine**: `ScreenState` (`Menu`, `Playing`, `Paused`, `Settings`, `LevelUp`, `GameOver { victory }`) transitions always clear active touch sticks via `touch.clear()` to eliminate stuck virtual controls.
3. **Platform Isolation**:
   - Android JNI focus-loss pause (`Java_dev_vex_neon_1core_MainActivity_neonOnPause`) is isolated in `src/main.rs` and paired with `quad.toml` + `android_java/immersive_inject.java`.
   - WASM audio stubs are isolated in `src/audio/mod.rs`.
   - Filesystem persistence is isolated in `src/persistence/save.rs`.
4. **Zero-Overhead Data Structures**: Fixed-capacity `Vec::with_capacity`, dense `swap_remove`, stack-allocated `TextBuf`, intrusive linked-list `SpatialGrid`, and conditional single-pass enemy compaction preserve cache locality without ECS indirection or `unsafe` Rust.
