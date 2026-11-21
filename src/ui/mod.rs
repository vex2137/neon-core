//! UI rendering subsystem: world renderer, HUD, hangar/shop menus, overlays, and gamepad focus helpers.

pub mod hud;
pub mod menus;
pub mod overlays;
pub mod world_render;

use macroquad::prelude::*;

pub const CYAN: Color = Color::new(5.0 / 255.0, 217.0 / 255.0, 232.0 / 255.0, 1.0);
pub const PINK: Color = Color::new(1.0, 42.0 / 255.0, 109.0 / 255.0, 1.0);
pub const GOLD: Color = Color::new(249.0 / 255.0, 200.0 / 255.0, 14.0 / 255.0, 1.0);
pub const GREEN: Color = Color::new(5.0 / 255.0, 1.0, 161.0 / 255.0, 1.0);
pub const PURPLE: Color = Color::new(184.0 / 255.0, 41.0 / 255.0, 234.0 / 255.0, 1.0);
pub const SLATE: Color = Color::new(72.0 / 255.0, 100.0 / 255.0, 138.0 / 255.0, 1.0);

pub fn draw_centered_text(text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size.max(1.0) as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, font_size, color);
}

pub fn draw_right_text(text: &str, rx: f32, y: f32, font_size: f32, color: Color) -> f32 {
    let dims = measure_text(text, None, font_size.max(1.0) as u16, 1.0);
    draw_text(text, rx - dims.width, y, font_size, color);
    dims.width
}

/// Checks if a virtual-space button was tapped or clicked and returns the relative X (`0.0..=1.0`).
pub fn ui_button_hit_x(vrect: Rect, scale: f32) -> Option<f32> {
    let touch_list = touches();
    for touch in &touch_list {
        if touch.phase == TouchPhase::Started {
            let vp = touch.position / scale;
            if vrect.contains(vp) {
                return Some(((vp.x - vrect.x) / vrect.w.max(1.0)).clamp(0.0, 1.0));
            }
        }
    }
    if touch_list.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let vp = vec2(mx / scale, my / scale);
        if vrect.contains(vp) {
            return Some(((vp.x - vrect.x) / vrect.w.max(1.0)).clamp(0.0, 1.0));
        }
    }
    None
}

/// Draws a virtual-space UI button with optional gamepad focus highlight border.
pub fn ui_button_v_focused(
    vrect: Rect,
    scale: f32,
    label: &str,
    sub: Option<&str>,
    border: Color,
    focused: bool,
) -> bool {
    let rx = vrect.x * scale;
    let ry = vrect.y * scale;
    let rw = vrect.w * scale;
    let rh = vrect.h * scale;

    let bg = if focused {
        Color::from_rgba(18, 30, 56, 248)
    } else {
        Color::from_rgba(11, 17, 32, 240)
    };
    draw_rectangle(rx, ry, rw, rh, bg);

    if focused {
        let pad = 3.0 * scale;
        draw_rectangle_lines(
            rx - pad,
            ry - pad,
            rw + pad * 2.0,
            rh + pad * 2.0,
            2.4 * scale,
            Color::from_rgba(255, 255, 255, 220),
        );
        draw_rectangle_lines(rx, ry, rw, rh, 2.6 * scale, border);
    } else {
        draw_rectangle_lines(rx, ry, rw, rh, 2.0 * scale, border);
    }

    if let Some(sub_txt) = sub {
        let t_sz = (if vrect.h < 50.0 { 14.5 } else { 17.0 }) * scale;
        let s_sz = (if vrect.h < 50.0 { 11.5 } else { 12.5 }) * scale;
        draw_centered_text(label, rx + rw * 0.5, ry + rh * 0.44, t_sz, border);
        draw_centered_text(sub_txt, rx + rw * 0.5, ry + rh * 0.80, s_sz, LIGHTGRAY);
    } else {
        let f_sz = (if vrect.h < 40.0 { 14.5 } else { 18.0 }) * scale;
        draw_centered_text(
            label,
            rx + rw * 0.5,
            ry + rh * 0.5 + 5.5 * scale,
            f_sz,
            border,
        );
    }

    ui_button_hit_x(vrect, scale).is_some()
}
