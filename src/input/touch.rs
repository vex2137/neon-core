//! Virtual touch sticks, touch ID tracking, and responsive mobile control geometry.

use crate::game::config::Viewport;
use crate::game::state::Settings;
use macroquad::math::{vec2, Rect, Vec2};
use macroquad::prelude::{Touch, TouchPhase};

pub const STICK_MAX_RADIUS: f32 = 52.0;
pub const DASH_BUTTON_RADIUS: f32 = 46.0;
pub const DASH_TOUCH_HITBOX: f32 = 58.0;

#[derive(Clone, Copy, Debug)]
pub struct Stick {
    pub touch_id: Option<u64>,
    pub start: Vec2,
    pub cur: Vec2,
    pub dir: Vec2,
}

impl Stick {
    pub const fn new() -> Self {
        Self {
            touch_id: None,
            start: Vec2::ZERO,
            cur: Vec2::ZERO,
            dir: Vec2::ZERO,
        }
    }

    pub fn clear(&mut self) {
        self.touch_id = None;
        self.dir = Vec2::ZERO;
    }
}

impl Default for Stick {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TouchState {
    pub left_stick: Stick,
    pub right_stick: Stick,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchActionFlags {
    pub pause_pressed: bool,
    pub dash_pressed: bool,
}

impl TouchState {
    pub fn clear(&mut self) {
        self.left_stick = Stick::new();
        self.right_stick = Stick::new();
    }

    /// Processes a frame's touch events in physical pixels, mapping them to virtual viewport units.
    pub fn process_touches(
        &mut self,
        touch_events: &[Touch],
        vp: Viewport,
        settings: Settings,
    ) -> TouchActionFlags {
        let mut actions = TouchActionFlags::default();
        let pause_rect = pause_button_rect(vp);
        let dash_pos = dash_button_center(vp);
        let stick_rest = left_stick_rest_center(vp);
        let expanded_pause = Rect::new(
            pause_rect.x - 8.0,
            pause_rect.y - 8.0,
            pause_rect.w + 16.0,
            pause_rect.h + 16.0,
        );

        for touch in touch_events {
            let tpos = touch.position / vp.scale;
            match touch.phase {
                TouchPhase::Started => {
                    if expanded_pause.contains(tpos) {
                        self.clear();
                        actions.pause_pressed = true;
                        return actions;
                    }
                    if tpos.distance_squared(dash_pos) <= DASH_TOUCH_HITBOX * DASH_TOUCH_HITBOX {
                        actions.dash_pressed = true;
                        continue;
                    }
                    if tpos.x < vp.vw * 0.52 && self.left_stick.touch_id.is_none() {
                        let start = if settings.fixed_stick {
                            stick_rest
                        } else {
                            vec2(
                                tpos.x.clamp(vp.safe_x + 42.0, vp.vw * 0.50),
                                tpos.y.clamp(84.0, vp.vh - 46.0),
                            )
                        };
                        self.left_stick.touch_id = Some(touch.id);
                        self.left_stick.start = start;
                        self.left_stick.cur = tpos;
                        let delta = tpos - start;
                        let len = delta.length();
                        self.left_stick.dir = if len > 3.0 {
                            (delta / len) * (len.min(STICK_MAX_RADIUS) / STICK_MAX_RADIUS)
                        } else {
                            Vec2::ZERO
                        };
                    } else if !settings.auto_aim && self.right_stick.touch_id.is_none() {
                        self.right_stick.touch_id = Some(touch.id);
                        self.right_stick.start = tpos;
                        self.right_stick.cur = tpos;
                        self.right_stick.dir = Vec2::ZERO;
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(touch.id) == self.left_stick.touch_id {
                        let delta = tpos - self.left_stick.start;
                        let len = delta.length();
                        self.left_stick.cur = if len > STICK_MAX_RADIUS {
                            self.left_stick.start + (delta / len) * STICK_MAX_RADIUS
                        } else {
                            tpos
                        };
                        self.left_stick.dir = if len > 3.0 {
                            (delta / len) * (len.min(STICK_MAX_RADIUS) / STICK_MAX_RADIUS)
                        } else {
                            Vec2::ZERO
                        };
                    } else if Some(touch.id) == self.right_stick.touch_id {
                        let delta = tpos - self.right_stick.start;
                        let len = delta.length();
                        self.right_stick.cur = if len > STICK_MAX_RADIUS {
                            self.right_stick.start + (delta / len) * STICK_MAX_RADIUS
                        } else {
                            tpos
                        };
                        self.right_stick.dir = if len > 4.0 { delta / len } else { Vec2::ZERO };
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(touch.id) == self.left_stick.touch_id {
                        self.left_stick.clear();
                    } else if Some(touch.id) == self.right_stick.touch_id {
                        self.right_stick.clear();
                    }
                }
            }
        }

        actions
    }
}

#[inline]
pub fn pause_button_rect(vp: Viewport) -> Rect {
    let size = if vp.vw < 700.0 { 38.0 } else { 42.0 };
    Rect::new(vp.vw - vp.safe_x - size, 12.0, size, size)
}

#[inline]
pub fn dash_button_center(vp: Viewport) -> Vec2 {
    vec2(vp.vw - vp.safe_x - 66.0, vp.vh - 74.0)
}

#[inline]
pub fn left_stick_rest_center(vp: Viewport) -> Vec2 {
    vec2(vp.safe_x + 68.0, vp.vh - 76.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_touch(id: u64, phase: TouchPhase, x: f32, y: f32) -> Touch {
        Touch {
            id,
            phase,
            position: vec2(x, y),
        }
    }

    #[test]
    fn dynamic_left_stick_tracks_id_clamps_radius_and_clears_on_cancel() {
        let vp = Viewport::from_dimensions(2400.0, 1080.0); // scale = 2.0, vw = 1200, vh = 540
        let settings = Settings::default();
        let mut ts = TouchState::default();

        // Start touch at virtual (200, 350) -> physical (400, 700)
        ts.process_touches(
            &[mk_touch(1, TouchPhase::Started, 400.0, 700.0)],
            vp,
            settings,
        );
        assert_eq!(ts.left_stick.touch_id, Some(1));
        assert!((ts.left_stick.start - vec2(200.0, 350.0)).length() < 1e-3);

        // Drag far right to virtual (400, 350) -> physical (800, 700)
        ts.process_touches(
            &[mk_touch(1, TouchPhase::Moved, 800.0, 700.0)],
            vp,
            settings,
        );
        assert!((ts.left_stick.dir - vec2(1.0, 0.0)).length() < 1e-3);
        assert!((ts.left_stick.cur - vec2(200.0 + STICK_MAX_RADIUS, 350.0)).length() < 1e-3);

        // Cancel clears stick completely
        ts.process_touches(
            &[mk_touch(1, TouchPhase::Cancelled, 800.0, 700.0)],
            vp,
            settings,
        );
        assert_eq!(ts.left_stick.touch_id, None);
        assert_eq!(ts.left_stick.dir, Vec2::ZERO);
    }

    #[test]
    fn dash_and_pause_touch_do_not_hijack_movement_stick() {
        let vp = Viewport::from_dimensions(2400.0, 1080.0);
        let settings = Settings::default();
        let mut ts = TouchState::default();

        let dc = dash_button_center(vp) * vp.scale;
        let act = ts.process_touches(
            &[mk_touch(5, TouchPhase::Started, dc.x, dc.y)],
            vp,
            settings,
        );
        assert!(act.dash_pressed);
        assert_eq!(ts.left_stick.touch_id, None);

        let pr = pause_button_rect(vp);
        let pc = vec2(pr.x + pr.w * 0.5, pr.y + pr.h * 0.5) * vp.scale;
        let act2 = ts.process_touches(
            &[mk_touch(6, TouchPhase::Started, pc.x, pc.y)],
            vp,
            settings,
        );
        assert!(act2.pause_pressed);
    }
}
