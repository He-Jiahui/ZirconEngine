#[path = "gamepad/axis/mod.rs"]
mod axis;
#[path = "gamepad/button/mod.rs"]
mod button;
#[path = "gamepad/device/mod.rs"]
mod device;
#[path = "gamepad/rumble/mod.rs"]
mod rumble;
#[path = "gamepad/value_scaling.rs"]
mod value_scaling;

pub use self::{
    axis::{
        GamepadAxis, GamepadAxisInput, GamepadAxisSettings, GamepadAxisState,
        GamepadAxisTransition, GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER,
        GAMEPAD_AXIS_DEADZONE_UPPER, GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER,
    },
    button::{
        GamepadButton, GamepadButtonAxisSettings, GamepadButtonSettings, GamepadButtonValueState,
        GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD, GAMEPAD_BUTTON_AXIS_HIGH, GAMEPAD_BUTTON_AXIS_LOW,
        GAMEPAD_BUTTON_PRESS_THRESHOLD, GAMEPAD_BUTTON_RELEASE_THRESHOLD,
    },
    device::{GamepadConnectionInfo, GamepadId},
    rumble::{GamepadRumbleIntensity, GamepadRumbleRequest},
};
