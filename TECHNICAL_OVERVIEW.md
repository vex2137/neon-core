# NEON CORE

A fast-paced neon twin-stick arena survival shooter built entirely in **Rust** with **Macroquad 0.4**, targeting **Android ARM64**, **Desktop**, and **WebAssembly**.

Fight through escalating waves of geometric enemies, chain combos, trigger tactical wave events, defeat multi-phase bosses, upgrade your ship mid-run, and spend collected shards in the Hangar for permanent progression.

---

## Screenshots

> Add gameplay screenshots or GIFs under `docs/media/` and link them below:
>
> - **Title & Hangar** — Ship selector, difficulty tiers, and permanent shard upgrades
> - **Mid-Wave Combat** — Neon arena grid, orbital saws, chain lightning, and dynamic touch controls
> - **Boss Encounter** — Multi-phase bosses, orbital hazards, and Overload Beacon objectives
> - **Level-Up Draft** — 3-card upgrade selection during combat

---

## Features

- **3 Playable Ships**:
  - **Vanguard** — Balanced interceptor with twin plasma cannons
  - **Phantom** — High-speed hull with shorter Dash cooldown and explosive mines dropped on dash
  - **Titan** — Heavy armored frame with plasma shielding and quad salvo fire
- **3 Difficulty Tiers**: Normal, Hard, and Nightmare (scaling enemy HP, speed, boss counts, and shard rewards)
- **7 Enemy Archetypes**: Chaser, Runner, Shooter, Tank, Splitter, Sniper, and multi-phase Bosses (plus Elite variants with enhanced durability and rewards)
- **Dynamic Wave Events & Objectives**: Swarm Rush, Sniper Ambush, Orbital Barrage, Boss Waves (every 3 waves), and interactive Overload Beacons
- **11 In-Run Level-Up Upgrades**: Rapid Fire, Heavy Rounds, Multi-Shot, Piercing Bolts, Thrusters, Hull Plating, Plasma Shield, Magnet Field, Orbital Saws, EMP Nova, and Chain Tesla
- **Permanent Hangar Shop**: 10 persistent upgrades purchased with Shards across runs
- **Responsive Cross-Platform Controls**:
  - **Mobile / Touch**: Dynamic or fixed left movement joystick, optional right aim stick, thumb-sized Dash button with cooldown arc, auto-aim toggle, safe-area inset padding, and Android immersive fullscreen with auto-pause on focus loss
  - **Desktop / Web**: WASD / Arrow movement, Mouse aim & fire (or Auto-Aim), Space/Shift Dash, keyboard upgrade shortcuts (`1`/`2`/`3`), and Escape/P pause
- **Procedural Audio Engine**: Zero external audio files — 13 synthesized WAV sound effects generated in memory at startup (with a zero-cost no-op shim on WebAssembly)
- **Resilient Save Persistence**: Versioned `v2` save format with 64-bit FNV-1a integrity checksum, atomic write-and-rename, `.bak` automatic recovery, and transparent migration from legacy `v1` CSV saves
- **Deterministic Simulation Core**: Seeded `XorShift64*` PRNG (`GameRng`) with automated regression and determinism test coverage

---

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Game Framework**: [Macroquad 0.4](https://github.com/not-fl3/macroquad)
- **Android Packaging**: `cargo-quad-apk` (Android API 26–35, `aarch64-linux-android`, package `dev.vex.neon_core`, custom immersive `MainActivity` JNI lifecycle hook)
- **Web Target**: `wasm32-unknown-unknown` (`--features demo`)
- **CI/CD**: GitHub Actions (`.github/workflows/rust-android.yml`)

---

## Supported Platforms

| Platform | Target / Packaging | Audio | Persistence |
| :--- | :--- | :--- | :--- |
| **Android (ARM64)** | `aarch64-linux-android` via `cargo-quad-apk` (minSdk 26, targetSdk 35) | Procedural WAV (`macroquad::audio`) | Atomic `v2` file in app data dir |
| **Linux / Windows / macOS** | Native binary (`cargo build --release`) | Procedural WAV (`macroquad::audio`) | Atomic `v2` file (`neon_core_save.dat`) |
| **WebAssembly (Demo)** | `wasm32-unknown-unknown --features demo` | No-op compile-time shim | In-memory session state |

---

## Build Instructions

See [`docs/BUILDING.md`](docs/BUILDING.md) for full prerequisite installation and troubleshooting details.

### Desktop (Native)

On Linux, install Macroquad's native development libraries first:

```bash
sudo apt-get install -y libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev
```

Run or build in release mode:

```bash
cargo run --release
# or produce binary only:
cargo build --release
```

### WebAssembly (Demo)

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --features demo
```

Output binary: `target/wasm32-unknown-unknown/release/neon_core.wasm` (compatible with `mq_js_bundle.js`).

### Android APK (`aarch64-linux-android`)

Requires Android SDK, NDK, and `cargo-quad-apk` (automatically built on every push via GitHub Actions):

```bash
rustup target add aarch64-linux-android
cargo install cargo-quad-apk
cargo quad-apk build --release
```

---

## Controls

### Desktop & WebAssembly

| Action | Input |
| :--- | :--- |
| Move | `W` `A` `S` `D` or Arrow Keys |
| Aim & Fire | Mouse cursor + Left Click (or automatic when **Auto-Aim** is enabled) |
| Dash | `Space` or `Left Shift` |
| Pause / Resume | `Esc` or `P` |
| Pick Upgrade Card | Click card or press `1`, `2`, `3` |

### Touch / Mobile (Android)

| Action | Input |
| :--- | :--- |
| Move | Left half of screen (Dynamic Joystick by default; Fixed mode toggleable in Settings) |
| Aim (Manual Mode) | Right-side Aim Stick when Auto-Aim is disabled |
| Dash | Bottom-right Dash button (with live cooldown ring) |
| Pause | Top-right Pause button (also triggers automatically when Android backgrounds the app) |

---

## Project Structure

```text
src/
  main.rs                 # Entry point, Macroquad Conf, Android JNI onPause hook, main loop
  rng.rs                  # Deterministic XorShift64* PRNG
  tests.rs                # Integration, determinism, regression, and release benchmark tests
  game/                   # Game orchestrator, virtual viewport config, screen state, run lifecycle
  player/                 # Ship blueprints, stats, movement, weapon salvos, dash/abilities
  enemies/                # 7 enemy archetypes, wave spawning, per-archetype AI & boss phases
  combat/                 # Projectiles, pure damage/crit/shield math, spatial-grid collisions
  world/                  # Fixed-capacity SpatialGrid, arena bounds, orbital hazards, Overload Beacon
  progression/            # Waves, wave events, XP/combo curves, upgrade draft, Hangar shop
  effects/                # Pooled particles/rings/arcs/texts, screen shake, zero-alloc TextBuf
  audio/                  # Procedural WAV synth and platform-gated SoundBank (native / WASM shim)
  input/                  # Multi-touch stick tracker and unified desktop/touch FrameInput
  persistence/            # Versioned v2 save format, FNV-1a checksum, atomic writes, v1 CSV migration
  performance/            # Rolling 120-frame PerfMetrics tracker
  ui/                     # World renderer, HUD, Hangar/Shop menus, Pause/Settings/LevelUp overlays
docs/
  ARCHITECTURE.md         # Module map, ownership boundaries, and frame lifecycle
  GAMEPLAY_SYSTEMS.md     # Ships, enemies, weapons, waves, events, and progression formulas
  PERFORMANCE.md          # Spatial grid, zero-alloc hot paths, and measured release benchmarks
  BUILDING.md             # Desktop, WebAssembly, Android APK, and CI build guide
  SAVE_FORMAT.md          # Save v2 wire format, checksum algorithm, atomic writes, v1 migration
```

---

## Testing & Benchmarks

Run formatting, static analysis, and the full test suite (49 unit, integration, regression, determinism, and benchmark tests):

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo check --target wasm32-unknown-unknown --features demo
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --features demo -- -D warnings
cargo test -- --nocapture
```

Run release-mode performance benchmarks:

```bash
cargo test --release benchmark_ -- --nocapture
```

See [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for measured benchmark results.

---

## License

Copyright © NEON CORE contributors. See repository settings or contact the maintainer for licensing terms.
