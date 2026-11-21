# Building & Running Neon Core

## Prerequisites

- **Rust** (stable toolchain, edition 2021) with `rustfmt` and `clippy` components.
- **Linux Desktop**: `libasound2-dev`, `libx11-dev`, `libxi-dev`, `libgl1-mesa-dev`.
- **WebAssembly**: `rustup target add wasm32-unknown-unknown`.
- **Android ARM64 APK**:
  - JDK 17 (Temurin)
  - Android SDK Platform 35 (`platforms;android-35`), Build-Tools `35.0.0`, NDK `25.2.9519653`
  - `rustup target add aarch64-linux-android`
  - `cargo-quad-apk`: `cargo install --git https://github.com/not-fl3/cargo-quad-apk --rev d411c8f --locked`

## Desktop Build & Run

```bash
cargo run --release
```

## Automated Checks & Tests

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test -- --nocapture
```

Run the heavy-combat simulation benchmark in release mode:

```bash
cargo test --release benchmark_heavy_combat_simulation_step -- --nocapture
```

## WebAssembly Demo Build

```bash
cargo build --release --target wasm32-unknown-unknown --features demo
```

The compiled module is written to `target/wasm32-unknown-unknown/release/neon_core.wasm` and loads via `mq_js_bundle.js` in a standard `<canvas id="glcanvas">` HTML host. Audio is compiled out via `default-features = false` on `wasm32` and replaced with a zero-cost no-op shim in `src/audio/mod.rs`.

## Android ARM64 APK Build

```bash
export NDK_HOME="$ANDROID_HOME/ndk/25.2.9519653"
export ANDROID_NDK_HOME="$NDK_HOME"
cargo quad-apk build --release
```

Output APK: `target/android-artifacts/release/apk/neon_core.apk`

- **Package Name**: `dev.vex.neon_core`
- **Target / Min SDK**: `35` / `26`
- **Orientation & Fullscreen**: `sensorLandscape` with `@style/NeonCoreFullscreen` (`android_res/values*/styles.xml`) and Android 11–15 `WindowInsetsController` immersive sticky injection (`quad.toml` + `android_java/immersive_inject.java`).
