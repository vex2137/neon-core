//! Input subsystems: touch sticks and unified keyboard/mouse/touch frame polling.

pub mod controls;
pub mod touch;

pub use controls::{poll_playing_input, FrameInput};
pub use touch::{
    dash_button_center, left_stick_rest_center, pause_button_rect, TouchState, DASH_BUTTON_RADIUS,
    STICK_MAX_RADIUS,
};
