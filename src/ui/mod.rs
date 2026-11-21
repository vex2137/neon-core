//! UI rendering subsystem: world renderer, HUD, hangar/shop menus, and overlays.

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

/// Draws a virtual-space UI button scaled to high-DPI physical pixels and checks touch/mouse press.
pub fn ui_button_v(vrect: Rect, scale: f32, label: &str, sub: Option<&str>, border: Color) -> bool {
    let rx = vrect.x * scale;
    let ry = vrect.y * scale;
    let rw = vrect.w * scale;
    let rh = vrect.h * scale;

    draw_rectangle(rx, ry, rw, rh, Color::from_rgba(11, 17, 32, 240));
    draw_rectangle_lines(rx, ry, rw, rh, 2.0 * scale, border);

    if let Some(sub_txt) = sub {
        let t_sz = (if vrect.h < 50.0 { 15.0 } else { 17.5 }) * scale;
        let s_sz = (if vrect.h < 50.0 { 12.0 } else { 13.0 }) * scale;
        draw_centered_text(label, rx + rw * 0.5, ry + rh * 0.44, t_sz, border);
        draw_centered_text(sub_txt, rx + rw * 0.5, ry + rh * 0.80, s_sz, LIGHTGRAY);
    } else {
        let f_sz = (if vrect.h < 40.0 { 15.5 } else { 19.0 }) * scale;
        draw_centered_text(
            label,
            rx + rw * 0.5,
            ry + rh * 0.5 + 6.0 * scale,
            f_sz,
            border,
        );
    }

    let touch_list = touches();
    for touch in &touch_list {
        if touch.phase == TouchPhase::Started && vrect.contains(touch.position / scale) {
            return true;
        }
    }
    if touch_list.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        return vrect.contains(vec2(mx / scale, my / scale));
    }
    false
}
