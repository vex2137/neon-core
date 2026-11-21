//! Core simulation capacities, arena dimensions, and virtual viewport scaling.

use macroquad::prelude::{screen_height, screen_width};

pub const VIRTUAL_HEIGHT: f32 = 540.0;
pub const ARENA_RADIUS: f32 = 2800.0;
pub const DEMO_MAX_WAVE: u32 = 8;

pub const MAX_BULLETS: usize = 520;
pub const MAX_EBULLETS: usize = 320;
pub const MAX_HAZARDS: usize = 28;
pub const MAX_MINES: usize = 18;
pub const MAX_ENEMIES: usize = 420;
pub const MAX_PARTICLES: usize = 640;
pub const MAX_RINGS: usize = 48;
pub const MAX_TEXTS: usize = 80;
pub const MAX_GEMS: usize = 380;
pub const MAX_ARCS: usize = 48;

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub sw: f32,
    pub sh: f32,
    pub scale: f32,
    pub vw: f32,
    pub vh: f32,
    pub safe_x: f32,
}

impl Viewport {
    #[inline]
    pub fn from_dimensions(raw_sw: f32, raw_sh: f32) -> Self {
        let sw = if raw_sw.is_finite() {
            raw_sw.max(1.0)
        } else {
            1280.0
        };
        let sh = if raw_sh.is_finite() {
            raw_sh.max(1.0)
        } else {
            720.0
        };
        let by_h = sh / VIRTUAL_HEIGHT;
        let by_w = sw / 640.0;
        let scale = by_h.min(by_w).max(0.45);
        let vw = sw / scale;
        let vh = sh / scale;
        let aspect = vw / vh.max(1.0);
        let safe_x = if aspect >= 1.95 {
            36.0
        } else if aspect >= 1.7 {
            18.0
        } else {
            12.0
        };
        Self {
            sw,
            sh,
            scale,
            vw,
            vh,
            safe_x,
        }
    }

    #[inline]
    pub fn current() -> Self {
        Self::from_dimensions(screen_width(), screen_height())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_scales_2400x1080_mobile_landscape_correctly() {
        let vp = Viewport::from_dimensions(2400.0, 1080.0);
        assert!((vp.scale - 2.0).abs() < 1e-4);
        assert!((vp.vw - 1200.0).abs() < 1e-3);
        assert!((vp.vh - 540.0).abs() < 1e-3);
        assert!((vp.safe_x - 36.0).abs() < 1e-4);
    }

    #[test]
    fn viewport_handles_invalid_dimensions_gracefully() {
        let vp = Viewport::from_dimensions(f32::NAN, 0.0);
        assert!(vp.scale.is_finite() && vp.scale >= 0.45);
        assert!(vp.vw.is_finite() && vp.vh.is_finite());
    }
}
