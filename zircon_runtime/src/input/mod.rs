//! Runtime input subsystem and protocol types.

mod module;
mod runtime;

pub use crate::core::framework::input::{
    ButtonInputState, FileDragDropEvent, GamepadAxis, GamepadAxisState, GamepadButton,
    GamepadConnectionInfo, GamepadId, ImeCursorArea, ImeCursorRange, ImeDeleteSurrounding,
    ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText, InputFrameSnapshot, MouseScrollUnit,
    MouseWheelEvent, TouchPhase, TouchPoint, WindowStatusEvent, WindowTheme,
    LEGACY_PIXEL_SCROLL_SCALE,
};
pub use crate::core::framework::input::{InputButton, InputEvent, InputEventRecord, InputSnapshot};
pub use module::{
    module_descriptor, InputConfig, InputModule, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME,
    INPUT_MODULE_NAME,
};
pub use runtime::{DefaultInputManager, InputDriver};

#[cfg(test)]
mod tests;
