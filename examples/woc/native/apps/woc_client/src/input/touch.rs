use super::gamepad::{stick_to_move_flags, GamepadMoveFlags, GamepadStickVector};

pub const TOUCH_JOYSTICK_DEADZONE: f64 = 0.22;
pub const MOVE_AUTORUN_REVEAL_THRESHOLD: f64 = 1.45;
pub const MOVE_AUTORUN_THRESHOLD: f64 = 2.05;
pub const CHAT_LONG_PRESS_MS: u64 = 420;
pub const RECENTER_DOUBLE_TAP_MS: u64 = 300;
pub const TOUCH_CAMERA_SENSITIVITY: f64 = 0.8;
pub const PINCH_ZOOM_DEADZONE_PX: f64 = 12.0;
pub const PINCH_ZOOM_SENSITIVITY: f64 = 0.035;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchInterfaceMode {
    Auto,
    Desktop,
    Touch,
}

pub fn map_touch_joystick(x: f64, y: f64, deadzone: Option<f64>) -> GamepadMoveFlags {
    stick_to_move_flags(x, y, deadzone.unwrap_or(TOUCH_JOYSTICK_DEADZONE))
}

pub fn is_move_autorun_push(y: f64, threshold: f64) -> bool {
    y <= -threshold
}

pub fn is_move_autorun_near(y: f64, threshold: f64) -> bool {
    y <= -threshold
}

pub fn touch_interface_mode_from_setting(value: f64) -> TouchInterfaceMode {
    if value >= 2.0 {
        TouchInterfaceMode::Touch
    } else if value >= 1.0 {
        TouchInterfaceMode::Desktop
    } else {
        TouchInterfaceMode::Auto
    }
}

pub fn resolve_touch_interface(mode: TouchInterfaceMode, auto_detected: bool) -> bool {
    match mode {
        TouchInterfaceMode::Auto => auto_detected,
        TouchInterfaceMode::Desktop => false,
        TouchInterfaceMode::Touch => true,
    }
}

pub fn is_chat_long_press(held_ms: u64, threshold_ms: u64) -> bool {
    held_ms >= threshold_ms
}

pub fn is_recenter_double_tap(
    previous_tap_ms: u64,
    now_ms: u64,
    moved: bool,
    threshold_ms: u64,
) -> bool {
    !moved && previous_tap_ms > 0 && now_ms.saturating_sub(previous_tap_ms) <= threshold_ms
}

pub fn map_touch_look_vector(x: f64, y: f64, deadzone: Option<f64>) -> GamepadStickVector {
    if x.hypot(y) < deadzone.unwrap_or(TOUCH_JOYSTICK_DEADZONE) {
        GamepadStickVector::default()
    } else {
        GamepadStickVector {
            x: x * TOUCH_CAMERA_SENSITIVITY,
            y: y * TOUCH_CAMERA_SENSITIVITY,
        }
    }
}

pub fn pinch_zoom_delta(
    previous_distance: f64,
    current_distance: f64,
    sensitivity: Option<f64>,
    deadzone_px: Option<f64>,
) -> f64 {
    let delta = current_distance - previous_distance;
    let absolute = delta.abs();
    let deadzone_px = deadzone_px.unwrap_or(PINCH_ZOOM_DEADZONE_PX);
    if absolute <= deadzone_px {
        return 0.0;
    }
    -delta.signum() * (absolute - deadzone_px) * sensitivity.unwrap_or(PINCH_ZOOM_SENSITIVITY)
}
