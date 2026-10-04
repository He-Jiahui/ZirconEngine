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
