mod axis;
mod input;
mod settings;
mod state;
mod transition;

pub use self::{
    axis::GamepadAxis,
    input::GamepadAxisInput,
    settings::{
        GamepadAxisSettings, GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER,
        GAMEPAD_AXIS_DEADZONE_UPPER, GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER,
    },
    state::GamepadAxisState,
    transition::GamepadAxisTransition,
};
