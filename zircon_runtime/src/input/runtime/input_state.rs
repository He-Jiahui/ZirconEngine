use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::input::{
    ButtonInputState, CursorGrabMode, CursorHostRequest, FileDragDropEvent, GamepadAxis,
    GamepadAxisState, GamepadAxisTransition, GamepadButton, GamepadButtonValueState, GamepadId,
    GamepadRumbleRequest, ImeDeleteSurrounding, ImeHostRequest, ImePreedit, InputButton,
    MouseScrollUnit, MouseWheelEvent, TouchPoint, WindowStatusEvent,
};

use super::event_buffer::{FrameEventBuffer, InputEventRecorder};

#[derive(Debug)]
pub(crate) struct InputState {
    pub(crate) cursor_position: [f32; 2],
    pub(crate) cursor_inside_window: bool,
    pub(crate) cursor_host_requests: Vec<CursorHostRequest>,
    pub(crate) cursor_host_requests_frame_start: usize,
    pub(crate) buttons: ButtonInputState<InputButton>,
    pub(crate) wheel_accumulator: f32,
    pub(crate) mouse_wheel_accumulator: [f32; 2],
    pub(crate) mouse_wheel_unit: MouseScrollUnit,
    pub(crate) mouse_wheel_events: Vec<MouseWheelEvent>,
    pub(crate) mouse_motion_accumulator: [f32; 2],
    pub(crate) active_touches: BTreeMap<u64, TouchPoint>,
    pub(crate) connected_gamepads: BTreeSet<GamepadId>,
    pub(crate) gamepad_axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
    pub(crate) gamepad_axis_transitions: Vec<GamepadAxisTransition>,
    gamepad_axis_transition_indices: HashMap<(GamepadId, GamepadAxis), usize>,
    pub(crate) gamepad_button_values: BTreeMap<(GamepadId, GamepadButton), f32>,
    pub(crate) gamepad_rumble_requests: Vec<GamepadRumbleRequest>,
    pub(crate) gamepad_rumble_requests_frame_start: usize,
    pub(crate) ime_enabled: bool,
    pub(crate) ime_preedit: Option<ImePreedit>,
    pub(crate) ime_commits: Vec<String>,
    pub(crate) ime_delete_surrounding: Vec<ImeDeleteSurrounding>,
    pub(crate) ime_host_requests: Vec<ImeHostRequest>,
    pub(crate) ime_host_requests_frame_start: usize,
    pub(crate) window_status_events: Vec<WindowStatusEvent>,
    pub(crate) file_drag_drop_events: Vec<FileDragDropEvent>,
    pub(super) frame_events: FrameEventBuffer,
    pub(super) event_recorder: InputEventRecorder,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            cursor_position: [0.0, 0.0],
            cursor_inside_window: false,
            cursor_host_requests: Vec::new(),
            cursor_host_requests_frame_start: 0,
            buttons: ButtonInputState::default(),
            wheel_accumulator: 0.0,
            mouse_wheel_accumulator: [0.0, 0.0],
            mouse_wheel_unit: MouseScrollUnit::Line,
            mouse_wheel_events: Vec::new(),
            mouse_motion_accumulator: [0.0, 0.0],
            active_touches: BTreeMap::new(),
            connected_gamepads: BTreeSet::new(),
            gamepad_axes: BTreeMap::new(),
            gamepad_axis_transitions: Vec::new(),
            gamepad_axis_transition_indices: HashMap::new(),
            gamepad_button_values: BTreeMap::new(),
            gamepad_rumble_requests: Vec::new(),
            gamepad_rumble_requests_frame_start: 0,
            ime_enabled: false,
            ime_preedit: None,
            ime_commits: Vec::new(),
            ime_delete_surrounding: Vec::new(),
            ime_host_requests: Vec::new(),
            ime_host_requests_frame_start: 0,
            window_status_events: Vec::new(),
            file_drag_drop_events: Vec::new(),
            frame_events: FrameEventBuffer::default(),
            event_recorder: InputEventRecorder::default(),
        }
    }
}

impl InputState {
    pub(crate) fn clear_gamepad_axis_transitions(&mut self) {
        self.gamepad_axis_transitions.clear();
        self.gamepad_axis_transition_indices.clear();
    }

    pub(crate) fn record_gamepad_axis_transition(
        &mut self,
        gamepad: GamepadId,
        axis: GamepadAxis,
        previous_value: f32,
        value: f32,
    ) {
        let key = (gamepad, axis);
        if let Some(index) = self.gamepad_axis_transition_indices.get(&key).copied() {
            self.gamepad_axis_transitions[index].value = value;
            return;
        }

        let index = self.gamepad_axis_transitions.len();
        self.gamepad_axis_transitions.push(GamepadAxisTransition {
            gamepad,
            axis,
            previous_value,
            value,
        });
        self.gamepad_axis_transition_indices.insert(key, index);
    }

    /// Losing interactive focus ends active controls but does not disconnect attached devices.
    pub(crate) fn clear_active_input_for_focus_loss(&mut self) {
        self.buttons.release_all();
        self.wheel_accumulator = 0.0;
        self.mouse_wheel_accumulator = [0.0, 0.0];
        self.mouse_wheel_unit = MouseScrollUnit::Line;
        self.mouse_wheel_events.clear();
        self.mouse_motion_accumulator = [0.0, 0.0];
        self.active_touches.clear();
        self.gamepad_button_values.clear();
        self.ime_enabled = false;
        self.ime_preedit = None;
        self.ime_host_requests.clear();
        self.ime_host_requests_frame_start = 0;
        self.ime_host_requests.push(ImeHostRequest::Disable);
        self.cursor_host_requests.clear();
        self.cursor_host_requests_frame_start = 0;
        self.cursor_host_requests
            .push(CursorHostRequest::set_grab_mode(CursorGrabMode::None));

        let appended_start = self.gamepad_axis_transitions.len();
        reset_gamepad_axes_for_focus_loss(
            std::mem::take(&mut self.gamepad_axes),
            &mut self.gamepad_axis_transitions,
        );
        self.index_appended_gamepad_axis_transitions(appended_start);
    }

    pub(crate) fn append_disconnected_axis_transitions(&mut self, disconnected_gamepad: GamepadId) {
        let appended_start = self.gamepad_axis_transitions.len();
        append_disconnected_axis_transitions(
            &self.gamepad_axes,
            &mut self.gamepad_axis_transitions,
            disconnected_gamepad,
        );
        self.index_appended_gamepad_axis_transitions(appended_start);
    }

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

    pub(crate) fn gamepad_button_value_states(&self) -> Vec<GamepadButtonValueState> {
        self.gamepad_button_values
            .iter()
            .map(|((gamepad, button), value)| GamepadButtonValueState {
                gamepad: *gamepad,
                button: *button,
                value: *value,
            })
            .collect()
    }

    fn index_appended_gamepad_axis_transitions(&mut self, appended_start: usize) {
        for index in appended_start..self.gamepad_axis_transitions.len() {
            let transition = self.gamepad_axis_transitions[index];
            self.gamepad_axis_transition_indices
                .entry((transition.gamepad, transition.axis))
                .or_insert(index);
        }
    }
}

fn reset_gamepad_axes_for_focus_loss(
    mut axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
    transitions: &mut Vec<GamepadAxisTransition>,
) {
    for transition in transitions.iter_mut() {
        let Some(previous_value) = axes.remove(&(transition.gamepad, transition.axis)) else {
            continue;
        };
        if previous_value != 0.0 {
            transition.value = 0.0;
        }
    }
    transitions.extend(
        axes.into_iter()
            .filter_map(|((gamepad, axis), previous_value)| {
                (previous_value != 0.0).then_some(GamepadAxisTransition {
                    gamepad,
                    axis,
                    previous_value,
                    value: 0.0,
                })
            }),
    );
}

fn append_disconnected_axis_transitions(
    axes: &BTreeMap<(GamepadId, GamepadAxis), f32>,
    transitions: &mut Vec<GamepadAxisTransition>,
    disconnected_gamepad: GamepadId,
) {
    transitions.extend(axes.iter().filter_map(|((gamepad, axis), value)| {
        (gamepad == &disconnected_gamepad && *value != 0.0).then_some(GamepadAxisTransition {
            gamepad: *gamepad,
            axis: *axis,
            previous_value: *value,
            value: 0.0,
        })
    }));
}

#[cfg(test)]
#[path = "tests/input_state.rs"]
mod tests;
