//! 定义平台输入经运行时归一化后的共享数据和服务契约：事件进入管理器，帧快照供动作求值与回放读取。
//! 设备采样、队列保留及动作求值的状态所有权位于 `crate::input::runtime`。

#[path = "button_input_state.rs"]
mod button_input_state;
#[path = "cursor.rs"]
mod cursor;
#[path = "event_retention/mod.rs"]
mod event_retention;
#[path = "file_drag_drop.rs"]
mod file_drag_drop;
#[path = "gamepad.rs"]
mod gamepad;
#[path = "ime.rs"]
mod ime;
#[path = "input_action.rs"]
mod input_action;
#[path = "input_action_context.rs"]
mod input_action_context;
#[path = "input_action_manager.rs"]
mod input_action_manager;
#[path = "input_action_map.rs"]
mod input_action_map;
#[path = "input_action_state.rs"]
mod input_action_state;
#[path = "input_batch.rs"]
mod input_batch;
#[path = "input_binding.rs"]
mod input_binding;
#[path = "input_button.rs"]
mod input_button;
#[path = "input_event.rs"]
mod input_event;
#[path = "input_event_record.rs"]
mod input_event_record;
#[path = "input_frame_snapshot.rs"]
mod input_frame_snapshot;
#[path = "input_manager.rs"]
mod input_manager;
#[path = "input_snapshot.rs"]
mod input_snapshot;
#[path = "module_identity.rs"]
mod module_identity;
#[path = "mouse_wheel.rs"]
mod mouse_wheel;
#[path = "touch.rs"]
mod touch;
#[path = "window_status.rs"]
mod window_status;

pub use button_input_state::ButtonInputState;
pub use cursor::{CursorGrabMode, CursorHostRequest, CursorPosition};
pub use event_retention::{
    InputEventQueueStatus, InputEventRecordingConfig, InputEventRecordingStatus,
    DEFAULT_INPUT_EVENT_RECORDING_CAPACITY,
};
pub use file_drag_drop::FileDragDropEvent;
pub use gamepad::{
    GamepadAxis, GamepadAxisInput, GamepadAxisSettings, GamepadAxisState, GamepadAxisTransition,
    GamepadButton, GamepadButtonAxisSettings, GamepadButtonSettings, GamepadButtonValueState,
    GamepadConnectionInfo, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest,
    GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER, GAMEPAD_AXIS_DEADZONE_UPPER,
    GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER, GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD,
    GAMEPAD_BUTTON_AXIS_HIGH, GAMEPAD_BUTTON_AXIS_LOW, GAMEPAD_BUTTON_PRESS_THRESHOLD,
    GAMEPAD_BUTTON_RELEASE_THRESHOLD,
};
pub use ime::{
    ImeCursorArea, ImeCursorRange, ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit,
    ImeSurroundingText,
};
pub use input_action::InputAction;
pub use input_action_context::InputActionContext;
pub use input_action_manager::InputActionManager;
pub use input_action_map::InputActionMap;
pub use input_action_state::InputActionState;
pub use input_batch::{InputBatchLimits, InputBatchRejection, InputBatchSize};
pub use input_binding::{InputAxisBinding, InputAxisDirection, InputBinding};
pub use input_button::InputButton;
pub use input_event::InputEvent;
pub use input_event_record::InputEventRecord;
pub use input_frame_snapshot::InputFrameSnapshot;
pub use input_manager::{
    GamepadInputRejection, InputEventAdmission, InputManager, QueuedGamepadRumbleRequest,
};
pub use input_snapshot::InputSnapshot;
pub use module_identity::INPUT_MODULE_NAME;
pub use mouse_wheel::{MouseScrollUnit, MouseWheelEvent, PIXEL_SCROLL_LINE_DELTA_SCALE};
pub use touch::{TouchPhase, TouchPoint};
pub use window_status::{WindowStatusEvent, WindowTheme};
