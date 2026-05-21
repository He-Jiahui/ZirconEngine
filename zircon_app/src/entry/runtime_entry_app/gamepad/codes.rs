use gilrs::{Axis, Button};
use zircon_runtime_interface::{
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1, ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1,
};

pub(super) fn button_code(button: Button) -> u32 {
    match button {
        Button::South => ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
        Button::East => ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
        Button::North => ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
        Button::West => ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
        Button::C => ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
        Button::Z => ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1,
        Button::LeftTrigger => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
        Button::LeftTrigger2 => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1,
        Button::RightTrigger => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1,
        Button::RightTrigger2 => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
        Button::Select => ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
        Button::Start => ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
        Button::Mode => ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1,
        Button::LeftThumb => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
        Button::RightThumb => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1,
        Button::DPadUp => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
        Button::DPadDown => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
        Button::DPadLeft => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
        Button::DPadRight => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
        Button::Unknown => ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1,
    }
}

pub(super) fn axis_code(axis: Axis) -> u32 {
    match axis {
        Axis::LeftStickX => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
        Axis::LeftStickY => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
        Axis::LeftZ => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1,
        Axis::RightStickX => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
        Axis::RightStickY => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1,
        Axis::RightZ => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
        Axis::DPadX => ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1,
        Axis::DPadY => ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
        Axis::Unknown => ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_codes_cover_common_gilrs_buttons() {
        assert_eq!(
            button_code(Button::South),
            ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1
        );
        assert_eq!(button_code(Button::East), ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1);
        assert_eq!(
            button_code(Button::North),
            ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1
        );
        assert_eq!(button_code(Button::West), ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1);
        assert_eq!(
            button_code(Button::LeftTrigger2),
            ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1
        );
        assert_eq!(
            button_code(Button::RightTrigger2),
            ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1
        );
        assert_eq!(
            button_code(Button::DPadUp),
            ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1
        );
        assert_eq!(
            button_code(Button::Unknown),
            ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1
        );
    }

    #[test]
    fn axis_codes_cover_sticks_triggers_and_dpad() {
        assert_eq!(
            axis_code(Axis::LeftStickX),
            ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1
        );
        assert_eq!(
            axis_code(Axis::LeftStickY),
            ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1
        );
        assert_eq!(axis_code(Axis::LeftZ), ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1);
        assert_eq!(
            axis_code(Axis::RightStickX),
            ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1
        );
        assert_eq!(
            axis_code(Axis::RightStickY),
            ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1
        );
        assert_eq!(axis_code(Axis::RightZ), ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1);
        assert_eq!(axis_code(Axis::DPadX), ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1);
        assert_eq!(axis_code(Axis::DPadY), ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1);
        assert_eq!(axis_code(Axis::Unknown), ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1);
    }
}
