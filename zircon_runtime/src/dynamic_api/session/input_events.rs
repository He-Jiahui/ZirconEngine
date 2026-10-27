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
    // Embedded Game input uses DOM codes; zircon_app retains FNV codes for
    // physical keys outside its legacy subset. Accept both transports without
    // deriving button identity from text, which is absent on key release.
    match key_code {
        8 => Some("Backspace"),
        9 => Some("Tab"),
        13 | 2_365_054_562 => Some("Enter"),
        16 => Some("Shift"),
        17 => Some("Control"),
        18 => Some("Alt"),
        19 | 1_157_218_093 => Some("Pause"),
        20 | 2_769_091_631 => Some("CapsLock"),
        27 | 3_082_514_982 => Some("Escape"),
        32 | 3_250_860_581 => Some("Space"),
        33 | 1_044_186_795 => Some("PageUp"),
        34 | 3_241_480_638 => Some("PageDown"),
        35 => Some("End"),
        36 => Some("Home"),
        37 => Some("ArrowLeft"),
        38 => Some("ArrowUp"),
        39 => Some("ArrowRight"),
        40 => Some("ArrowDown"),
        44 | 3_614_257_180 => Some("PrintScreen"),
        45 | 254_900_552 => Some("Insert"),
        46 => Some("Delete"),
        48 | 734_172_594 => Some("0"),
        49 | 750_950_213 => Some("1"),
        50 | 700_617_356 => Some("2"),
        51 | 717_394_975 => Some("3"),
        52 | 667_062_118 => Some("4"),
        53 | 683_839_737 => Some("5"),
        54 | 633_506_880 => Some("6"),
        55 | 650_284_499 => Some("7"),
        56 | 868_393_546 => Some("8"),
        57 | 885_171_165 => Some("9"),
        65 => Some("A"),
        66 | 791_340_490 => Some("B"),
        67 => Some("C"),
        68 => Some("D"),
        69 | 707_452_395 => Some("E"),
        70 | 724_230_014 => Some("F"),
        71 | 741_007_633 => Some("G"),
        72 | 623_564_300 => Some("H"),
        73 | 640_341_919 => Some("I"),
        74 | 657_119_538 => Some("J"),
        75 | 673_897_157 => Some("K"),
        76 | 556_453_824 => Some("L"),
        77 | 573_231_443 => Some("M"),
        78 | 590_009_062 => Some("N"),
        79 | 606_786_681 => Some("O"),
        80 | 1_026_227_156 => Some("P"),
        81 | 1_043_004_775 => Some("Q"),
        82 | 1_059_782_394 => Some("R"),
        83 => Some("S"),
        84 | 959_116_680 => Some("T"),
        85 | 975_894_299 => Some("U"),
        86 => Some("V"),
        87 => Some("W"),
        88 => Some("X"),
        89 => Some("Y"),
        90 => Some("Z"),
        91 | 1_231_322_581 => Some("Meta"),
        92 | 1_051_083_118 => Some("Meta"),
        93 | 231_243_091 => Some("ContextMenu"),
        96 | 1_910_265_404 => Some("0"),
        97 | 1_927_043_023 => Some("1"),
        98 | 1_943_820_642 => Some("2"),
        99 | 1_960_598_261 => Some("3"),
        100 | 1_843_154_928 => Some("4"),
        101 | 1_859_932_547 => Some("5"),
        102 | 1_876_710_166 => Some("6"),
        103 | 1_893_487_785 => Some("7"),
        104 | 2_044_486_356 => Some("8"),
        105 | 2_061_263_975 => Some("9"),
        106 | 3_890_594_748 => Some("*"),
        107 | 3_369_747_263 => Some("+"),
        109 | 2_262_086_360 => Some("-"),
        110 | 524_308_083 => Some("."),
        111 | 66_885_981 => Some("/"),
        112 | 332_577_688 => Some("F1"),
        113 | 382_910_545 => Some("F2"),
        114 | 366_132_926 => Some("F3"),
        115 | 416_465_783 => Some("F4"),
        116 | 399_688_164 => Some("F5"),
        117 | 450_021_021 => Some("F6"),
        118 | 433_243_402 => Some("F7"),
        119 | 215_134_355 => Some("F8"),
        120 | 198_356_736 => Some("F9"),
        121 | 3_703_400_824 => Some("F10"),
        122 | 3_720_178_443 => Some("F11"),
        123 | 3_736_956_062 => Some("F12"),
        124 | 3_753_733_681 => Some("F13"),
        125 | 3_770_511_300 => Some("F14"),
        126 | 3_787_288_919 => Some("F15"),
        127 | 3_804_066_538 => Some("F16"),
        128 | 3_820_844_157 => Some("F17"),
        129 | 3_569_179_872 => Some("F18"),
        130 | 3_585_957_491 => Some("F19"),
        131 | 1_321_523_379 => Some("F20"),
        132 | 1_304_745_760 => Some("F21"),
        133 | 1_355_078_617 => Some("F22"),
        134 | 1_338_300_998 => Some("F23"),
        135 | 1_388_633_855 => Some("F24"),
        144 | 2_979_892_988 => Some("NumLock"),
        145 | 4_019_901_635 => Some("ScrollLock"),
        186 | 257_323_198 => Some(";"),
        187 | 24_087_439 => Some("="),
        188 | 1_706_424_088 => Some(","),
        189 | 3_388_260_431 => Some("-"),
        190 | 2_267_317_284 => Some("."),
        191 | 1_898_928_778 => Some("/"),
        192 | 1_462_265_726 => Some("`"),
        219 | 1_169_940_618 => Some("["),
        220 | 1_258_159_639 => Some("\\"),
        221 | 896_166_103 => Some("]"),
        222 | 1_522_423_415 => Some("'"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/input_events.rs"]
mod tests;

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

// 先借用验证 UTF-8；只有有效载荷才分配 Runtime 需要持有的字符串。
fn owned_utf8_payload(payload: &[u8]) -> Option<String> {
    std::str::from_utf8(payload).ok().map(str::to_owned)
}

pub(in crate::dynamic_api::session) fn ime_surrounding_text(
    event: ZrRuntimeEventV1,
    payload: &[u8],
) -> Result<ImeSurroundingText, ZrStatus> {
    let value = match owned_utf8_payload(payload) {
        Some(value) => value,
        None => return Err(invalid_argument(b"invalid runtime ime payload")),
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

#[cfg(test)]
#[path = "input_events/tests/borrowed_utf8_tests.rs"]
mod borrowed_utf8_tests;
