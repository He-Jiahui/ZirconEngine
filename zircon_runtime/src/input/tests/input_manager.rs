use crate::core::framework::input::InputManager;

use crate::input::{
    ButtonInputState, CursorGrabMode, CursorHostRequest, DefaultInputManager, FileDragDropEvent,
    GamepadAxis, GamepadAxisSettings, GamepadButton, GamepadButtonAxisSettings,
    GamepadConnectionInfo, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest, ImeCursorArea,
    ImeCursorRange, ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText,
    InputButton, InputEvent, InputFrameSnapshot, MouseScrollUnit, MouseWheelEvent, TouchPhase,
    WindowStatusEvent, WindowTheme, PIXEL_SCROLL_LINE_DELTA_SCALE,
};

#[path = "input_manager/event_buffer.rs"]
mod event_buffer;
#[path = "input_manager/frame_state.rs"]
mod frame_state;
#[path = "input_manager/host_requests.rs"]
mod host_requests;
#[path = "input_manager/touch_gamepad.rs"]
mod touch_gamepad;
