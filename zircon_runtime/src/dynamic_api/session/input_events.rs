use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrStatus, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_C_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1, ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1, ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_START_V1, ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1,
    ZR_RUNTIME_IME_CURSOR_HIDDEN_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
    ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_MOVED_V1, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    ZR_RUNTIME_WINDOW_BOOL_FALSE_V1, ZR_RUNTIME_WINDOW_BOOL_TRUE_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
};

use crate::core::framework::input::{
    GamepadAxis, GamepadButton, ImeCursorArea, ImeCursorRange, ImeSurroundingText, InputButton,
    MouseScrollUnit, TouchPhase, WindowTheme,
};

use super::status::invalid_argument;

pub(in crate::dynamic_api::session) fn input_button(button: u32) -> Option<InputButton> {
    match button {
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => Some(InputButton::MouseLeft),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => Some(InputButton::MouseRight),
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => Some(InputButton::MouseMiddle),
        _ => None,
    }
}

pub(in crate::dynamic_api::session) fn mouse_scroll_unit(
    unit: u32,
) -> Result<Option<MouseScrollUnit>, ZrStatus> {
    match unit {
        0 => Ok(None),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1 => Ok(Some(MouseScrollUnit::Line)),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1 => Ok(Some(MouseScrollUnit::Pixel)),
        _ => Err(invalid_argument(b"unknown runtime mouse wheel unit")),
    }
}

pub(in crate::dynamic_api::session) fn touch_phase(phase: u32) -> Option<TouchPhase> {
    match phase {
        ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => Some(TouchPhase::Started),
        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => Some(TouchPhase::Moved),
        ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 => Some(TouchPhase::Ended),
        ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => Some(TouchPhase::Cancelled),
        _ => None,
    }
}

pub(in crate::dynamic_api::session) fn keyboard_logical_key(
    key_code: u32,
    _text: Option<&str>,
) -> Option<String> {
    keyboard_button_name(key_code).map(str::to_string)
}

fn keyboard_button_name(key_code: u32) -> Option<&'static str> {
    match key_code {
        16 => Some("Shift"),
        17 => Some("Control"),
        18 => Some("Alt"),
        _ => None,
    }
}

pub(in crate::dynamic_api::session) fn ime_cursor(
    event: ZrRuntimeEventV1,
) -> Option<ImeCursorRange> {
    if event.key_code == ZR_RUNTIME_IME_CURSOR_HIDDEN_V1
        || event.scan_code == ZR_RUNTIME_IME_CURSOR_HIDDEN_V1
    {
        None
    } else {
        Some(ImeCursorRange::new(
            event.key_code as usize,
            event.scan_code as usize,
        ))
    }
}

pub(in crate::dynamic_api::session) fn ime_cursor_area(
    event: ZrRuntimeEventV1,
) -> Option<ImeCursorArea> {
    if event.x.is_finite() && event.y.is_finite() && event.size.width > 0 && event.size.height > 0 {
        Some(ImeCursorArea::new(
            event.x,
            event.y,
            event.size.width as f32,
            event.size.height as f32,
        ))
    } else {
        None
    }
}

pub(in crate::dynamic_api::session) fn ime_surrounding_text(
    event: ZrRuntimeEventV1,
    payload: &[u8],
) -> Result<ImeSurroundingText, ZrStatus> {
    let value = match String::from_utf8(payload.to_vec()) {
        Ok(value) => value,
        Err(_) => return Err(invalid_argument(b"invalid runtime ime payload")),
    };
    let cursor = event.key_code as usize;
    let anchor = event.scan_code as usize;
    if cursor > value.len()
        || anchor > value.len()
        || !value.is_char_boundary(cursor)
        || !value.is_char_boundary(anchor)
    {
        return Err(invalid_argument(b"invalid runtime ime surrounding text"));
    }
    Ok(ImeSurroundingText::new(value, cursor, anchor))
}

pub(in crate::dynamic_api::session) fn window_bool(value: u32) -> Option<bool> {
    match value {
        ZR_RUNTIME_WINDOW_BOOL_FALSE_V1 => Some(false),
        ZR_RUNTIME_WINDOW_BOOL_TRUE_V1 => Some(true),
        _ => None,
    }
}

pub(in crate::dynamic_api::session) fn window_theme(theme: u32) -> WindowTheme {
    match theme {
        ZR_RUNTIME_WINDOW_THEME_LIGHT_V1 => WindowTheme::Light,
        ZR_RUNTIME_WINDOW_THEME_DARK_V1 => WindowTheme::Dark,
        _ => WindowTheme::Unknown,
    }
}

pub(in crate::dynamic_api::session) fn window_scale_factor(value: f32) -> Option<f32> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

pub(in crate::dynamic_api::session) fn gamepad_button(button: u32) -> GamepadButton {
    match button {
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1 => GamepadButton::South,
        ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1 => GamepadButton::East,
        ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1 => GamepadButton::North,
        ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1 => GamepadButton::West,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1 => GamepadButton::LeftTrigger,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1 => GamepadButton::LeftTrigger2,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1 => GamepadButton::RightTrigger,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1 => GamepadButton::RightTrigger2,
        ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1 => GamepadButton::Select,
        ZR_RUNTIME_GAMEPAD_BUTTON_START_V1 => GamepadButton::Start,
        ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1 => GamepadButton::Mode,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1 => GamepadButton::LeftThumb,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1 => GamepadButton::RightThumb,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1 => GamepadButton::DPadUp,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1 => GamepadButton::DPadDown,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1 => GamepadButton::DPadLeft,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1 => GamepadButton::DPadRight,
        ZR_RUNTIME_GAMEPAD_BUTTON_C_V1
        | ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1
        | ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1 => GamepadButton::Other(button as u16),
        _ => GamepadButton::Other(button as u16),
    }
}

pub(in crate::dynamic_api::session) fn gamepad_axis(axis: u32) -> GamepadAxis {
    match axis {
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1 => GamepadAxis::LeftStickX,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1 => GamepadAxis::LeftStickY,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1 => GamepadAxis::LeftZ,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1 => GamepadAxis::RightStickX,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1 => GamepadAxis::RightStickY,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1 => GamepadAxis::RightZ,
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1 => GamepadAxis::DPadX,
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1 => GamepadAxis::DPadY,
        ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1 => GamepadAxis::Other(axis as u16),
        _ => GamepadAxis::Other(axis as u16),
    }
}

pub(in crate::dynamic_api::session) fn nonzero_u16(value: u32) -> Option<u16> {
    u16::try_from(value).ok().filter(|value| *value != 0)
}
