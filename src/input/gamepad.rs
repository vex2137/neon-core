//! Universal cross-platform gamepad support:
//! - Desktop & WASM: `gilrs` polling with hot-plug resilience and optional desktop rumble.
//! - Android (OTG / Bluetooth): JNI native callbacks (`neonOnGamepadAxis`, `neonOnGamepadButton`)
//!   feeding a thread-safe `AndroidGamepadState`.

use crate::game::state::Settings;
#[cfg(any(not(target_arch = "wasm32"), test))]
use macroquad::math::vec2;
use macroquad::math::Vec2;
use macroquad::prelude::{is_key_pressed, KeyCode};

#[cfg(target_os = "android")]
use std::sync::Mutex;

/// Radial deadzone with smooth rescaling from `0.0` at the deadzone edge to `1.0` at full deflection.
#[inline]
pub fn apply_radial_deadzone(raw: Vec2, deadzone: f32) -> Vec2 {
    if !raw.x.is_finite() || !raw.y.is_finite() {
        return Vec2::ZERO;
    }
    let dz = deadzone.clamp(0.05, 0.30);
    let len = raw.length();
    if len <= dz {
        return Vec2::ZERO;
    }
    let scaled = ((len - dz) / (1.0 - dz)).clamp(0.0, 1.0);
    (raw / len) * scaled
}

/// Raw normalized state sampled from either `gilrs` or the Android JNI bridge.
#[derive(Clone, Copy, Debug, Default)]
pub struct RawGamepadState {
    pub connected: bool,
    pub left_stick: Vec2, // +X right, +Y down (Macroquad world/screen convention)
    pub right_stick: Vec2, // +X right, +Y down
    pub dpad: Vec2,       // +X right, +Y down
    pub lt: f32,          // 0..=1
    pub rt: f32,          // 0..=1
    pub south: bool,      // A / Cross
    pub east: bool,       // B / Circle
    pub west: bool,       // X / Square
    pub north: bool,      // Y / Triangle
    pub lb: bool,         // L1 / LB
    pub rb: bool,         // R1 / RB
    pub start: bool,      // Start / Options / Menu
    pub select: bool,     // Select / Share / View
}

/// Per-frame processed gamepad snapshot (analog vectors after deadzone + clean edge-triggered actions).
#[derive(Clone, Copy, Debug, Default)]
pub struct GamepadSnapshot {
    #[allow(dead_code)]
    pub connected: bool,
    pub move_dir: Vec2,
    pub aim_dir: Option<Vec2>,
    pub fire_held: bool,
    pub dash_pressed: bool,
    pub pause_pressed: bool,

    // UI navigation edges
    pub nav_up: bool,
    pub nav_down: bool,
    pub nav_left: bool,
    pub nav_right: bool,
    pub confirm_pressed: bool,
    pub back_pressed: bool,
    pub prev_tab_pressed: bool,
    pub next_tab_pressed: bool,
    pub any_activity: bool,
}

/// Unified menu/overlay navigation input merging keyboard shortcuts and gamepad buttons.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuNavInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub back: bool,
    pub prev_tab: bool,
    pub next_tab: bool,
}

impl MenuNavInput {
    pub fn from_gamepad_and_keyboard(gp: &GamepadSnapshot) -> Self {
        Self {
            up: gp.nav_up || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
            down: gp.nav_down || is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
            left: gp.nav_left || is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
            right: gp.nav_right || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
            confirm: gp.confirm_pressed
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space),
            back: gp.back_pressed || is_key_pressed(KeyCode::Escape),
            prev_tab: gp.prev_tab_pressed || is_key_pressed(KeyCode::Q),
            next_tab: gp.next_tab_pressed || is_key_pressed(KeyCode::E),
        }
    }

    #[inline]
    pub fn any_pressed(self) -> bool {
        self.up
            || self.down
            || self.left
            || self.right
            || self.confirm
            || self.back
            || self.prev_tab
            || self.next_tab
    }
}

// ============================================================================
// Android JNI Native Gamepad Bridge (thread-safe, zero unsafe blocks)
// ============================================================================

#[cfg(any(target_os = "android", test))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AndroidGamepadState {
    pub connected: bool,
    pub lx: f32,
    pub ly: f32,
    pub rx: f32,
    pub ry: f32,
    pub lt: f32,
    pub rt: f32,
    pub hat_x: f32,
    pub hat_y: f32,
    pub buttons: [bool; 14],
}

#[cfg(any(target_os = "android", test))]
impl AndroidGamepadState {
    pub fn set_axis(&mut self, axis_code: i32, value: f32) {
        if !value.is_finite() {
            return;
        }
        let v = value.clamp(-1.0, 1.0);
        self.connected = true;
        match axis_code {
            0 => self.lx = v,
            1 => self.ly = v,
            2 => self.rx = v,
            3 => self.ry = v,
            4 => self.lt = v.clamp(0.0, 1.0),
            5 => self.rt = v.clamp(0.0, 1.0),
            6 => self.hat_x = v,
            7 => self.hat_y = v,
            _ => {}
        }
    }

    pub fn set_button(&mut self, key_code: i32, pressed: bool) {
        if (0..14).contains(&key_code) {
            self.connected = true;
            self.buttons[key_code as usize] = pressed;
        }
    }

    pub fn to_raw(self) -> RawGamepadState {
        let mut dpad = vec2(self.hat_x, self.hat_y);
        if self.buttons[10] {
            dpad.y -= 1.0;
        }
        if self.buttons[11] {
            dpad.y += 1.0;
        }
        if self.buttons[12] {
            dpad.x -= 1.0;
        }
        if self.buttons[13] {
            dpad.x += 1.0;
        }
        if dpad.length_squared() > 1.0 {
            dpad = dpad.normalize();
        }

        RawGamepadState {
            connected: self.connected,
            left_stick: vec2(self.lx, self.ly),
            right_stick: vec2(self.rx, self.ry),
            dpad,
            lt: if self.buttons[6] { 1.0 } else { self.lt },
            rt: if self.buttons[7] { 1.0 } else { self.rt },
            south: self.buttons[0],
            east: self.buttons[1],
            west: self.buttons[2],
            north: self.buttons[3],
            lb: self.buttons[4],
            rb: self.buttons[5],
            start: self.buttons[8],
            select: self.buttons[9],
        }
    }
}

#[cfg(target_os = "android")]
static ANDROID_GAMEPAD: Mutex<AndroidGamepadState> = Mutex::new(AndroidGamepadState {
    connected: false,
    lx: 0.0,
    ly: 0.0,
    rx: 0.0,
    ry: 0.0,
    lt: 0.0,
    rt: 0.0,
    hat_x: 0.0,
    hat_y: 0.0,
    buttons: [false; 14],
});

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_dev_vex_neon_1core_MainActivity_neonOnGamepadAxis(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    axis_code: i32,
    value: f32,
) {
    if let Ok(mut guard) = ANDROID_GAMEPAD.lock() {
        guard.set_axis(axis_code, value);
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_dev_vex_neon_1core_MainActivity_neonOnGamepadButton(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    key_code: i32,
    pressed: u8,
) {
    if let Ok(mut guard) = ANDROID_GAMEPAD.lock() {
        guard.set_button(key_code, pressed != 0);
    }
}

// ============================================================================
// Cross-Platform Gamepad Manager (Desktop/WASM `gilrs` + Android JNI)
// ============================================================================

pub struct GamepadManager {
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    gilrs: Option<gilrs::Gilrs>,
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    active_id: Option<gilrs::GamepadId>,
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    rumble_effect: Option<gilrs::ff::Effect>,

    prev_dash_combo: bool,
    prev_pause: bool,
    prev_confirm: bool,
    prev_back: bool,
    prev_lb: bool,
    prev_rb: bool,
    last_nav_x: i8,
    last_nav_y: i8,
    nav_repeat_x: f32,
    nav_repeat_y: f32,
    pub last_snapshot: GamepadSnapshot,
}

impl GamepadManager {
    pub fn new() -> Self {
        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        let gilrs = gilrs::Gilrs::new().ok();

        Self {
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            gilrs,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            active_id: None,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            rumble_effect: None,
            prev_dash_combo: false,
            prev_pause: false,
            prev_confirm: false,
            prev_back: false,
            prev_lb: false,
            prev_rb: false,
            last_nav_x: 0,
            last_nav_y: 0,
            nav_repeat_x: 0.0,
            nav_repeat_y: 0.0,
            last_snapshot: GamepadSnapshot::default(),
        }
    }

    /// Headless constructor that skips OS hardware enumeration (used in unit/integration tests).
    #[cfg(test)]
    pub fn headless() -> Self {
        Self {
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            gilrs: None,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            active_id: None,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            rumble_effect: None,
            prev_dash_combo: false,
            prev_pause: false,
            prev_confirm: false,
            prev_back: false,
            prev_lb: false,
            prev_rb: false,
            last_nav_x: 0,
            last_nav_y: 0,
            nav_repeat_x: 0.0,
            nav_repeat_y: 0.0,
            last_snapshot: GamepadSnapshot::default(),
        }
    }

    pub fn poll(&mut self, dt: f32, settings: Settings) -> GamepadSnapshot {
        let raw = self.sample_raw();
        let snap = self.process_raw(raw, dt, settings);
        self.last_snapshot = snap;
        snap
    }

    /// Pure edge-detection & deadzone processor (testable directly without hardware).
    pub fn process_raw(
        &mut self,
        raw: RawGamepadState,
        dt: f32,
        settings: Settings,
    ) -> GamepadSnapshot {
        let dz = settings.deadzone_f32();
        let left = apply_radial_deadzone(raw.left_stick, dz);
        let right = apply_radial_deadzone(raw.right_stick, dz);

        let mut move_dir = left + raw.dpad;
        if move_dir.length_squared() > 1.0 {
            move_dir = move_dir.normalize();
        }
        let aim_dir = if right.length_squared() > 0.0001 {
            Some(right.normalize())
        } else {
            None
        };

        let rt_down = raw.rt > 0.35;
        let lt_down = raw.lt > 0.35;
        let fire_held = rt_down || raw.rb || raw.south;
        let dash_now = rt_down || lt_down || raw.rb || raw.south || raw.west || raw.north;
        let dash_pressed = dash_now && !self.prev_dash_combo;
        self.prev_dash_combo = dash_now;

        let pause_down = raw.start || raw.select;
        let pause_pressed = pause_down && !self.prev_pause;
        self.prev_pause = pause_down;

        let confirm_pressed = raw.south && !self.prev_confirm;
        self.prev_confirm = raw.south;

        let back_pressed = raw.east && !self.prev_back;
        self.prev_back = raw.east;

        let prev_tab_pressed = raw.lb && !self.prev_lb;
        self.prev_lb = raw.lb;

        let next_tab_pressed = raw.rb && !self.prev_rb;
        self.prev_rb = raw.rb;

        // Repeatable discrete menu navigation from D-Pad or left stick
        let nav_x: i8 = if raw.dpad.x < -0.5 || left.x < -0.55 {
            -1
        } else if raw.dpad.x > 0.5 || left.x > 0.55 {
            1
        } else {
            0
        };
        let nav_y: i8 = if raw.dpad.y < -0.5 || left.y < -0.55 {
            -1
        } else if raw.dpad.y > 0.5 || left.y > 0.55 {
            1
        } else {
            0
        };

        let mut nav_left = false;
        let mut nav_right = false;
        if nav_x != 0 {
            if nav_x != self.last_nav_x {
                self.nav_repeat_x = 0.28;
                nav_left = nav_x < 0;
                nav_right = nav_x > 0;
            } else {
                self.nav_repeat_x -= dt;
                if self.nav_repeat_x <= 0.0 {
                    self.nav_repeat_x = 0.13;
                    nav_left = nav_x < 0;
                    nav_right = nav_x > 0;
                }
            }
        } else {
            self.nav_repeat_x = 0.0;
        }
        self.last_nav_x = nav_x;

        let mut nav_up = false;
        let mut nav_down = false;
        if nav_y != 0 {
            if nav_y != self.last_nav_y {
                self.nav_repeat_y = 0.28;
                nav_up = nav_y < 0;
                nav_down = nav_y > 0;
            } else {
                self.nav_repeat_y -= dt;
                if self.nav_repeat_y <= 0.0 {
                    self.nav_repeat_y = 0.13;
                    nav_up = nav_y < 0;
                    nav_down = nav_y > 0;
                }
            }
        } else {
            self.nav_repeat_y = 0.0;
        }
        self.last_nav_y = nav_y;

        let any_activity = raw.connected
            && (move_dir.length_squared() > 0.01
                || aim_dir.is_some()
                || fire_held
                || pause_pressed
                || confirm_pressed
                || back_pressed
                || prev_tab_pressed
                || next_tab_pressed
                || nav_up
                || nav_down
                || nav_left
                || nav_right);

        GamepadSnapshot {
            connected: raw.connected,
            move_dir,
            aim_dir,
            fire_held,
            dash_pressed,
            pause_pressed,
            nav_up,
            nav_down,
            nav_left,
            nav_right,
            confirm_pressed,
            back_pressed,
            prev_tab_pressed,
            next_tab_pressed,
            any_activity,
        }
    }

    /// Triggers short controller vibration when supported and enabled (graceful no-op otherwise).
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn trigger_rumble(&mut self, intensity: f32, duration_ms: u32, enabled: bool) {
        if !enabled || !intensity.is_finite() || intensity <= 0.0 || duration_ms == 0 {
            return;
        }
        use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
        let (Some(gilrs), Some(id)) = (self.gilrs.as_mut(), self.active_id) else {
            return;
        };
        if !gilrs.gamepad(id).is_ff_supported() {
            return;
        }
        let mag = (intensity.clamp(0.0, 1.0) * u16::MAX as f32) as u16;
        let ticks = Ticks::from_ms(duration_ms.min(600));
        if let Ok(eff) = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong { magnitude: mag },
                scheduling: Replay {
                    play_for: ticks,
                    ..Default::default()
                },
                ..Default::default()
            })
            .gamepads(&[id])
            .finish(gilrs)
        {
            let _ = eff.play();
            self.rumble_effect = Some(eff);
        }
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    pub fn trigger_rumble(&mut self, _intensity: f32, _duration_ms: u32, _enabled: bool) {}

    #[cfg(target_os = "android")]
    fn sample_raw(&mut self) -> RawGamepadState {
        if let Ok(guard) = ANDROID_GAMEPAD.lock() {
            guard.to_raw()
        } else {
            RawGamepadState::default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn sample_raw(&mut self) -> RawGamepadState {
        RawGamepadState::default()
    }

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    fn sample_raw(&mut self) -> RawGamepadState {
        use gilrs::{Axis, Button, EventType};

        let Some(gilrs) = self.gilrs.as_mut() else {
            return RawGamepadState::default();
        };

        // Drain all queued hot-plug and input events so internal state is fresh.
        while let Some(ev) = gilrs.next_event() {
            match ev.event {
                EventType::Disconnected => {
                    if self.active_id == Some(ev.id) {
                        self.active_id = None;
                    }
                }
                _ => {
                    self.active_id = Some(ev.id);
                }
            }
        }

        // Fall back to the first connected gamepad if active_id is unset or disconnected.
        let id = match self.active_id {
            Some(id) if gilrs.gamepad(id).is_connected() => id,
            _ => {
                let first = gilrs
                    .gamepads()
                    .find(|(_, gp)| gp.is_connected())
                    .map(|(id, _)| id);
                self.active_id = first;
                match first {
                    Some(id) => id,
                    None => return RawGamepadState::default(),
                }
            }
        };

        let gp = gilrs.gamepad(id);
        let lx = gp.value(Axis::LeftStickX);
        // gilrs uses +Y up; invert to match Macroquad screen/world (+Y down).
        let ly = -gp.value(Axis::LeftStickY);
        let rx = gp.value(Axis::RightStickX);
        let ry = -gp.value(Axis::RightStickY);

        let mut dpad = Vec2::ZERO;
        if gp.is_pressed(Button::DPadUp) {
            dpad.y -= 1.0;
        }
        if gp.is_pressed(Button::DPadDown) {
            dpad.y += 1.0;
        }
        if gp.is_pressed(Button::DPadLeft) {
            dpad.x -= 1.0;
        }
        if gp.is_pressed(Button::DPadRight) {
            dpad.x += 1.0;
        }
        let hat_x = gp.value(Axis::DPadX);
        let hat_y = -gp.value(Axis::DPadY);
        if hat_x.abs() > 0.25 {
            dpad.x = hat_x;
        }
        if hat_y.abs() > 0.25 {
            dpad.y = hat_y;
        }
        if dpad.length_squared() > 1.0 {
            dpad = dpad.normalize();
        }

        let lt_btn = gp
            .button_data(Button::LeftTrigger2)
            .map(|d| d.value())
            .unwrap_or(0.0);
        let rt_btn = gp
            .button_data(Button::RightTrigger2)
            .map(|d| d.value())
            .unwrap_or(0.0);
        let lt_axis = gp.value(Axis::LeftZ).max(0.0);
        let rt_axis = gp.value(Axis::RightZ).max(0.0);

        RawGamepadState {
            connected: true,
            left_stick: vec2(lx, ly),
            right_stick: vec2(rx, ry),
            dpad,
            lt: lt_btn.max(lt_axis).clamp(0.0, 1.0),
            rt: rt_btn.max(rt_axis).clamp(0.0, 1.0),
            south: gp.is_pressed(Button::South),
            east: gp.is_pressed(Button::East),
            west: gp.is_pressed(Button::West),
            north: gp.is_pressed(Button::North),
            lb: gp.is_pressed(Button::LeftTrigger),
            rb: gp.is_pressed(Button::RightTrigger),
            start: gp.is_pressed(Button::Start) || gp.is_pressed(Button::Mode),
            select: gp.is_pressed(Button::Select),
        }
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radial_deadzone_filters_drift_and_rescales_smoothly() {
        assert_eq!(apply_radial_deadzone(vec2(0.10, 0.05), 0.15), Vec2::ZERO);
        assert_eq!(apply_radial_deadzone(vec2(f32::NAN, 0.5), 0.15), Vec2::ZERO);

        let mid = apply_radial_deadzone(vec2(0.575, 0.0), 0.15);
        assert!((mid.x - 0.5).abs() < 1e-3);
        assert!(mid.y.abs() < 1e-5);

        let full = apply_radial_deadzone(vec2(1.0, 0.0), 0.20);
        assert!((full.x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn android_gamepad_state_maps_axes_hat_and_buttons() {
        let mut ag = AndroidGamepadState::default();
        ag.set_axis(0, 0.8);
        ag.set_axis(1, -0.6);
        ag.set_axis(2, 0.0);
        ag.set_axis(3, 1.0);
        ag.set_axis(5, 0.9);
        ag.set_button(0, true); // BUTTON_A
        ag.set_button(8, true); // START

        let raw = ag.to_raw();
        assert!(raw.connected);
        assert!((raw.left_stick.x - 0.8).abs() < 1e-5);
        assert!((raw.right_stick.y - 1.0).abs() < 1e-5);
        assert!(raw.rt > 0.85);
        assert!(raw.south);
        assert!(raw.start);
    }
}
