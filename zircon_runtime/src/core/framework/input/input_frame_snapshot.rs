use serde::{Deserialize, Serialize};

use super::{
    ButtonInputState, CursorHostRequest, FileDragDropEvent, GamepadAxisState,
    GamepadAxisTransition, GamepadButtonValueState, GamepadId, GamepadRumbleRequest,
    ImeDeleteSurrounding, ImeHostRequest, ImePreedit, InputButton, MouseScrollUnit,
    MouseWheelEvent, TouchPoint, WindowStatusEvent,
};

/// 帧级状态视图：按住和连接状态跨帧持续，边沿、增量与通知由 `begin_frame` 划定。
/// 宿主请求字段只是本帧新增视图；实际交付应使用管理器的排空接口。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputFrameSnapshot {
    pub cursor_position: [f32; 2],
    pub cursor_inside_window: bool,
    pub cursor_host_requests: Vec<CursorHostRequest>,
    pub buttons: ButtonInputState<InputButton>,
    pub wheel_accumulator: f32,
    // TODO: [CR-FRAMEWORK-INPUT-0001] 确定混合行/像素滚动时聚合值的单位契约；当前管理器直接相加并仅保存最后一个事件的单位，需核实消费方应使用逐事件列表还是归一化聚合。
    pub mouse_wheel_accumulator: [f32; 2],
    pub mouse_wheel_unit: MouseScrollUnit,
    pub mouse_wheel_events: Vec<MouseWheelEvent>,
    pub mouse_motion_accumulator: [f32; 2],
    pub active_touches: Vec<TouchPoint>,
    pub connected_gamepads: Vec<GamepadId>,
    pub gamepad_axes: Vec<GamepadAxisState>,
    pub gamepad_axis_transitions: Vec<GamepadAxisTransition>,
    pub gamepad_button_values: Vec<GamepadButtonValueState>,
    pub gamepad_rumble_requests: Vec<GamepadRumbleRequest>,
    pub ime_enabled: bool,
    pub ime_preedit: Option<ImePreedit>,
    pub ime_commits: Vec<String>,
    pub ime_delete_surrounding: Vec<ImeDeleteSurrounding>,
    pub ime_host_requests: Vec<ImeHostRequest>,
    pub window_status_events: Vec<WindowStatusEvent>,
    pub file_drag_drop_events: Vec<FileDragDropEvent>,
}

impl Default for InputFrameSnapshot {
    fn default() -> Self {
        Self {
            cursor_position: [0.0, 0.0],
            cursor_inside_window: false,
            cursor_host_requests: Vec::new(),
            buttons: ButtonInputState::default(),
            wheel_accumulator: 0.0,
            mouse_wheel_accumulator: [0.0, 0.0],
            mouse_wheel_unit: MouseScrollUnit::Line,
            mouse_wheel_events: Vec::new(),
            mouse_motion_accumulator: [0.0, 0.0],
            active_touches: Vec::new(),
            connected_gamepads: Vec::new(),
            gamepad_axes: Vec::new(),
            gamepad_axis_transitions: Vec::new(),
            gamepad_button_values: Vec::new(),
            gamepad_rumble_requests: Vec::new(),
            ime_enabled: false,
            ime_preedit: None,
            ime_commits: Vec::new(),
            ime_delete_surrounding: Vec::new(),
            ime_host_requests: Vec::new(),
            window_status_events: Vec::new(),
            file_drag_drop_events: Vec::new(),
        }
    }
}
