use std::collections::{BTreeMap, BTreeSet};

use crate::input::{
    ButtonInputState, FileDragDropEvent, GamepadAxis, GamepadAxisState, GamepadId,
    ImeDeleteSurrounding, ImeHostRequest, ImePreedit, InputButton, InputEvent, InputEventRecord,
    MouseScrollUnit, MouseWheelEvent, TouchPoint, WindowStatusEvent,
};

#[derive(Debug)]
pub(crate) struct InputState {
    pub(crate) cursor_position: [f32; 2],
    pub(crate) cursor_inside_window: bool,
    pub(crate) buttons: ButtonInputState<InputButton>,
    pub(crate) wheel_accumulator: f32,
    pub(crate) mouse_wheel_accumulator: [f32; 2],
    pub(crate) mouse_wheel_unit: MouseScrollUnit,
    pub(crate) mouse_wheel_events: Vec<MouseWheelEvent>,
    pub(crate) mouse_motion_accumulator: [f32; 2],
    pub(crate) active_touches: BTreeMap<u64, TouchPoint>,
    pub(crate) connected_gamepads: BTreeSet<GamepadId>,
    pub(crate) gamepad_axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
    pub(crate) ime_enabled: bool,
    pub(crate) ime_preedit: Option<ImePreedit>,
    pub(crate) ime_commits: Vec<String>,
    pub(crate) ime_delete_surrounding: Vec<ImeDeleteSurrounding>,
    pub(crate) ime_host_requests: Vec<ImeHostRequest>,
    pub(crate) window_status_events: Vec<WindowStatusEvent>,
    pub(crate) file_drag_drop_events: Vec<FileDragDropEvent>,
    pub(crate) events: Vec<InputEvent>,
    pub(crate) records: Vec<InputEventRecord>,
    pub(crate) next_sequence: u64,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            cursor_position: [0.0, 0.0],
            cursor_inside_window: false,
            buttons: ButtonInputState::default(),
            wheel_accumulator: 0.0,
            mouse_wheel_accumulator: [0.0, 0.0],
            mouse_wheel_unit: MouseScrollUnit::Line,
            mouse_wheel_events: Vec::new(),
            mouse_motion_accumulator: [0.0, 0.0],
            active_touches: BTreeMap::new(),
            connected_gamepads: BTreeSet::new(),
            gamepad_axes: BTreeMap::new(),
            ime_enabled: false,
            ime_preedit: None,
            ime_commits: Vec::new(),
            ime_delete_surrounding: Vec::new(),
            ime_host_requests: Vec::new(),
            window_status_events: Vec::new(),
            file_drag_drop_events: Vec::new(),
            events: Vec::new(),
            records: Vec::new(),
            next_sequence: 0,
        }
    }
}

impl InputState {
    pub(crate) fn gamepad_axis_states(&self) -> Vec<GamepadAxisState> {
        self.gamepad_axes
            .iter()
            .map(|((gamepad, axis), value)| GamepadAxisState {
                gamepad: *gamepad,
                axis: *axis,
                value: *value,
            })
            .collect()
    }
}
