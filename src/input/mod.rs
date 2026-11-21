//! Input subsystems: virtual touch sticks, cross-platform gamepad, and unified frame polling.

pub mod controls;
pub mod gamepad;
pub mod touch;

#[cfg(test)]
pub use controls::merge_playing_inputs;
pub use controls::{poll_playing_input, FrameInput};
#[cfg(test)]
pub use gamepad::RawGamepadState;
pub use gamepad::{GamepadManager, MenuNavInput};
pub use touch::{
    dash_button_center, left_stick_rest_center, pause_button_rect, TouchState, DASH_BUTTON_RADIUS,
    STICK_MAX_RADIUS,
};
