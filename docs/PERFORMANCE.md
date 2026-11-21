# NEON CORE — Performance Engineering

This document describes the hot-path architecture, memory discipline, rendering optimizations, runtime profiling infrastructure, and measured benchmark results for **NEON CORE**.

## 1. Frame Budget & Target Envelopes

| Target Platform | Target Frame Rate | Frame Budget | Release Artifact Size |
| :--- | :--- | :--- | :--- |
| Android ARM64 (`aarch64-linux-android`, API 26–35) | 60 / 120 FPS | 16.67 ms / 8.33 ms | Built in CI (`cargo-quad-apk`) |
| Linux Desktop (`x86_64-unknown-linux-gnu`) | 60+ FPS | 16.67 ms | `1.6 MB` (`target/release/neon_core`) |
| WebAssembly Demo (`wasm32-unknown-unknown --features demo`) | 60 FPS | 16.67 ms | `645 KB` (`neon_core.wasm`) |

Release profile (`Cargo.toml`):
- `opt-level = 3`
- `lto = "thin"`
- `codegen-units = 1`
- `strip = "debuginfo"`

## 2. Spatial Partitioning (`src/world/spatial_grid.rs`)

Collision detection, auto-aim targeting, chain-lightning propagation, and EMP Nova blast queries use a flat, fixed-capacity intrusive linked-list spatial hash grid:

- **Arena coverage**: `3400 x 3400` world units partitioned into `130`-unit cells (`27 x 27 = 729` cells).
- **Zero per-cell `Vec` allocations**: `cell_head: Vec<i32>` stores the first enemy index per cell, and `next: Vec<i32>` stores the intrusive linked list across all enemies.
- **Reused buffers**: `rebuild(&[Enemy])` fills `cell_head` with `-1` and resizes `next` only if enemy count exceeds previous high-water capacity. No heap allocation occurs during steady-state gameplay.
- **Boundary & NaN safety**: `cell_coord(v)` maps non-finite values to `0` and clamps coordinates to `[0, GRID_COLS - 1]`. Queries clamp the bounding box `[min_cx..=max_cx]` and `[min_cy..=max_cy]` so no out-of-bounds cell index can occur.
- **Rebuild cadence**: Rebuilt once per frame right before collision and targeting phases, and immediately invalidated/rebuilt only if enemy indices change via compaction.

## 3. Allocation & Hot-Path Optimizations

| System | Previous Bottleneck | Refactored Implementation |
| :--- | :--- | :--- |
| **Damage & pickup popups** (`src/effects/floating_text.rs`, `src/combat/damage.rs`) | Heap `String` allocated via `format!()` on every hit and pickup | Fixed-size stack buffer `TextBuf<32>` / `TextBuf<48>` implementing `core::fmt::Write` with safe UTF-8 truncation (`0` heap allocations) |
| **HUD rendering** (`src/ui/hud.rs`) | Multiple `format!()` `String` allocations per frame (HP, shield, XP, score, wave, combo, timer, shards) | Reusable stack `TextBuf<48>` / `TextBuf<96>` buffers (`0` heap allocations per HUD frame) |
| **Player hull geometry** (`src/ui/world_render.rs`) | `Vec<Vec2>` heap allocation every frame for ship hull vertices | Stack-allocated `[Vec2; 4]` / `[Vec2; 5]` / `[Vec2; 6]` fixed arrays |
| **Chain Tesla targeting** (`src/game/mod.rs`) | `Vec::with_capacity(8)` allocated on each chain trigger | Fixed stack array `[usize; 12]` with length counter (`0` heap allocations) |
| **Enemy compaction** (`src/game/mod.rs`) | Unconditional `enemies.retain(|e| e.alive)` called twice every frame | Deferred dirty-flag compaction (`enemies_dirty`), retaining only when at least one enemy died during the tick |
| **Entity removal** (`src/game/mod.rs`, `src/combat/collisions.rs`, `src/effects/mod.rs`) | Standard linear removal shifts | `swap_remove()` on bullets, enemy bullets, mines, XP gems, hazards, particles, rings, arcs, and floating texts |
| **Pre-allocated pools** (`src/game/session.rs`, `src/effects/mod.rs`) | Incremental growth spikes on wave start | `Vec::with_capacity(...)` pre-sized pools (`enemies: 260`, `bullets: 320`, `enemy_bullets: 220`, `gems: 360`, `particles: 480`, `rings: 64`, `arcs: 48`, `texts: 64`) |
| **Dash Ready ring** (`src/ui/hud.rs`) | 32 `draw_line` calls + 64 trig evaluations per frame when Dash was ready | Single `draw_circle_lines()` call when cooldown is ready; segmented arc only while recharging |
| **Arena background grid** (`src/ui/world_render.rs`) | Iterating full world bounds | Viewport-culled grid lines iterating strictly over visible world `[left..right]` and `[top..bottom]` |

## 4. Measured Release Benchmarks

Benchmarks are compiled as real Rust tests in `src/tests.rs` and executed in release mode via:

```bash
cargo test --release benchmark_ -- --nocapture
```

Measured on the local Linux x86_64 environment (`rustc 1.100.0`, `release` profile with `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`):

| Benchmark | Workload | Measured Result | Notes |
| :--- | :--- | :--- | :--- |
| **`heavy_combat_300_enemies`** | 240 full simulation frames (`dt = 1/60s`) with 300 active enemies, 120 enemy bullets, player auto-salvos, `SpatialGrid` rebuilds, orbital saws, EMP Nova, Chain Tesla, hazards, and XP gems | **6.47 ms total** (**27.0 µs / frame** average) | **0.16%** of a 16.67 ms 60 FPS frame budget (`89` kills, `53` occupied grid cells) |
| **`SpatialGrid` vs Linear Scan** | 200 rounds × 200 bullet queries against 400 distributed enemies (40,000 queries) | Linear: **29,761.6 µs** vs `SpatialGrid`: **1,645.6 µs** | **18.09× faster** with identical hit count (`4,400` hits) |
| **Stack `TextBuf` vs Heap `String`** | 50,000 formatting iterations (`"WAVE 12 | 123456"`) | Heap `String`: **7,466.2 µs** vs `TextBuf<48>`: **5,827.2 µs** | **1.28× faster** and **0 heap allocations** (eliminates allocator contention and fragmentation on mobile) |

> Note: Benchmark numbers reflect CPU simulation and formatting costs without GPU driver submission. GPU draw costs on mobile are kept low via viewport culling, capped particle/effect pools, and reduced HUD draw calls.

## 5. Runtime Diagnostics (`src/performance/metrics.rs`)

`PerfMetrics` records a zero-allocation 120-frame rolling window (`[f32; 120]`) tracking:
- Current FPS, average frame time (ms), and peak frame time (ms)
- Active entity counts (enemies, bullets, enemy bullets, gems, particles, occupied grid cells)

When enabled via the Settings overlay (**Perf HUD** toggle), a compact bottom-left performance strip renders live metrics using stack `TextBuf` formatting so toggling diagnostics adds negligible overhead.
