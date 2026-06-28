//! Runtime input subsystem and protocol types.

mod module;
mod runtime;

pub use crate::core::framework::input::{
    ButtonInputState, CursorGrabMode, CursorHostRequest, CursorPosition, FileDragDropEvent,
    GamepadAxis, GamepadAxisInput, GamepadAxisSettings, GamepadAxisState, GamepadAxisTransition,
    GamepadButton, GamepadButtonAxisSettings, GamepadButtonSettings, GamepadButtonValueState,
    GamepadConnectionInfo, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest, ImeCursorArea,
    ImeCursorRange, ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText,
    InputAction, InputActionContext, InputActionManager, InputActionMap, InputActionState,
    InputAxisBinding, InputAxisDirection, InputBinding, InputFrameSnapshot, MouseScrollUnit,
    MouseWheelEvent, TouchPhase, TouchPoint, WindowStatusEvent, WindowTheme,
    GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER, GAMEPAD_AXIS_DEADZONE_UPPER,
    GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER, GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD,
    GAMEPAD_BUTTON_AXIS_HIGH, GAMEPAD_BUTTON_AXIS_LOW, GAMEPAD_BUTTON_PRESS_THRESHOLD,
    GAMEPAD_BUTTON_RELEASE_THRESHOLD, PIXEL_SCROLL_LINE_DELTA_SCALE,
};
pub use crate::core::framework::input::{InputButton, InputEvent, InputEventRecord, InputSnapshot};
pub use module::{
    module_descriptor, module_descriptor_with_config, InputConfig, InputModule,
    INPUT_ACTION_MANAGER_NAME, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME, INPUT_MODULE_NAME,
};
pub use runtime::{
    DefaultInputActionManager, DefaultInputManager, InputActionEvaluator, InputDriver,
    InputRecording, InputRecordingFrame, InputReplayCursor, InputReplayFrameReport,
};

#[cfg(test)]
mod tests;
