use std::collections::{BTreeMap, BTreeSet};

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

        reset_gamepad_axes_for_focus_loss(
            std::mem::take(&mut self.gamepad_axes),
            &mut self.gamepad_axis_transitions,
        );
    }

    pub(crate) fn append_disconnected_axis_transitions(&mut self, disconnected_gamepad: GamepadId) {
        append_disconnected_axis_transitions(
            &self.gamepad_axes,
            &mut self.gamepad_axis_transitions,
            disconnected_gamepad,
        );
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
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const BENCHMARK_SAMPLES: usize = 11;
    const FOCUS_RESET_ITERATIONS: usize = 8;
    const FOCUS_RESET_AXIS_COUNT: usize = 1_024;
    const FOCUS_RESET_TRANSITION_COUNT: usize = FOCUS_RESET_AXIS_COUNT / 2;
    const DISCONNECT_ITERATIONS: usize = 64;
    const DISCONNECT_AXIS_COUNT: usize = 4_096;

    #[test]
    fn runtime56_recovery_batch_indexed_focus_loss_axis_reset_preserves_retired_transition_order_and_values(
    ) {
        let axes = BTreeMap::from([
            (axis_key(1, GamepadAxis::LeftStickX), 0.5),
            (axis_key(1, GamepadAxis::LeftStickY), 0.0),
            (axis_key(2, GamepadAxis::RightStickX), -0.75),
            (axis_key(3, GamepadAxis::DPadX), 1.0),
        ]);
        let transitions = vec![
            transition(2, GamepadAxis::RightStickX, 0.25, -0.75),
            transition(1, GamepadAxis::LeftStickX, 0.0, 0.5),
            transition(1, GamepadAxis::LeftStickX, 0.25, 0.5),
            transition(9, GamepadAxis::RightStickY, 0.0, 0.25),
        ];
        let mut retired = transitions.clone();
        let mut optimized = transitions;

        retired_reset_gamepad_axes_for_focus_loss(axes.clone(), &mut retired);
        reset_gamepad_axes_for_focus_loss(axes, &mut optimized);

        assert_eq!(optimized, retired);
        assert_eq!(optimized[0].value, 0.0);
        assert_eq!(optimized[1].value, 0.0);
        assert_eq!(optimized[2].value, 0.5);
        assert_eq!(optimized[3].value, 0.25);
        assert_eq!(
            optimized.last(),
            Some(&transition(3, GamepadAxis::DPadX, 1.0, 0.0))
        );
    }

    #[test]
    fn runtime56_recovery_batch_indexed_focus_loss_axis_reset_source_contract() {
        let source = include_str!("input_state.rs");
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let reset = production
            .split_once("fn reset_gamepad_axes_for_focus_loss")
            .expect("indexed focus-loss axis reset")
            .1
            .split_once("fn append_disconnected_axis_transitions")
            .expect("focus-loss reset helper end")
            .0;

        assert!(reset.contains("for transition in transitions.iter_mut()"));
        assert!(reset.contains("axes.remove(&(transition.gamepad, transition.axis))"));
        assert!(reset.contains("transitions.extend("));
        assert!(!reset.contains(".iter_mut()\n                .find("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_indexed_focus_loss_axis_reset_release_benchmark() {
        let axes = benchmark_axes(FOCUS_RESET_AXIS_COUNT);
        let transitions = (0..FOCUS_RESET_TRANSITION_COUNT)
            .map(|index| transition(0, GamepadAxis::Other(index as u16), -0.5, 0.5))
            .collect::<Vec<_>>();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_focus_reset(
                    &axes,
                    &transitions,
                    retired_reset_gamepad_axes_for_focus_loss,
                ));
                optimized_samples.push(measure_focus_reset(
                    &axes,
                    &transitions,
                    reset_gamepad_axes_for_focus_loss,
                ));
            } else {
                optimized_samples.push(measure_focus_reset(
                    &axes,
                    &transitions,
                    reset_gamepad_axes_for_focus_loss,
                ));
                retired_samples.push(measure_focus_reset(
                    &axes,
                    &transitions,
                    retired_reset_gamepad_axes_for_focus_loss,
                ));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let retired_linear_probe_upper_bound =
            FOCUS_RESET_AXIS_COUNT * FOCUS_RESET_TRANSITION_COUNT;
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_INDEXED_FOCUS_LOSS_AXIS_RESET_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={FOCUS_RESET_ITERATIONS} \
axes={FOCUS_RESET_AXIS_COUNT} existing_transitions={FOCUS_RESET_TRANSITION_COUNT} \
retired_linear_probe_upper_bound={retired_linear_probe_upper_bound} \
optimized_tree_removals={FOCUS_RESET_TRANSITION_COUNT} \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(25),
            "indexed focus-loss reset must reduce P95 by at least 75%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    #[test]
    fn runtime56_recovery_batch_allocation_free_gamepad_disconnect_transitions_preserve_retired_output(
    ) {
        let axes = BTreeMap::from([
            (axis_key(1, GamepadAxis::LeftStickX), 0.5),
            (axis_key(1, GamepadAxis::LeftStickY), 0.0),
            (axis_key(1, GamepadAxis::RightStickX), -0.75),
            (axis_key(2, GamepadAxis::DPadX), 1.0),
        ]);
        let initial = vec![transition(9, GamepadAxis::RightStickY, 0.0, 0.25)];
        let mut retired = initial.clone();
        let mut optimized = initial;

        retired_append_disconnected_axis_transitions(&axes, &mut retired, GamepadId(1));
        append_disconnected_axis_transitions(&axes, &mut optimized, GamepadId(1));

        assert_eq!(optimized, retired);
        assert_eq!(optimized.len(), 3);
        assert!(optimized
            .iter()
            .all(|transition| transition.gamepad != GamepadId(2)));
    }

    #[test]
    fn runtime56_recovery_batch_allocation_free_gamepad_disconnect_transitions_source_contract() {
        let input_state_source = include_str!("input_state.rs");
        let input_state_production = input_state_source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let append = input_state_production
            .split_once("fn append_disconnected_axis_transitions")
            .expect("direct disconnect transition helper")
            .1;
        let manager_source = include_str!("default_input_manager.rs");
        let connection = manager_source
            .split_once("InputEvent::GamepadConnection(info)")
            .expect("gamepad connection arm")
            .1
            .split_once("InputEvent::GamepadButton")
            .expect("gamepad connection arm end")
            .0;

        assert!(append.contains("transitions.extend("));
        assert!(!append.contains("collect::<Vec<_>>()"));
        assert!(connection.contains("state.append_disconnected_axis_transitions(info.gamepad)"));
        assert!(!connection.contains("let disconnected_axis_transitions"));
        assert!(!connection.contains("extend(disconnected_axis_transitions)"));
        assert!(!connection.contains("collect::<Vec<_>>()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_allocation_free_gamepad_disconnect_transitions_release_benchmark() {
        let axes = benchmark_axes(DISCONNECT_AXIS_COUNT);
        let target_gamepad = GamepadId(0);
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_disconnect_append(
                    &axes,
                    target_gamepad,
                    retired_append_disconnected_axis_transitions,
                ));
                optimized_samples.push(measure_disconnect_append(
                    &axes,
                    target_gamepad,
                    append_disconnected_axis_transitions,
                ));
            } else {
                optimized_samples.push(measure_disconnect_append(
                    &axes,
                    target_gamepad,
                    append_disconnected_axis_transitions,
                ));
                retired_samples.push(measure_disconnect_append(
                    &axes,
                    target_gamepad,
                    retired_append_disconnected_axis_transitions,
                ));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let matching_axes = DISCONNECT_AXIS_COUNT / 2;
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_ALLOCATION_FREE_GAMEPAD_DISCONNECT_TRANSITIONS_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={DISCONNECT_ITERATIONS} \
axes={DISCONNECT_AXIS_COUNT} matching_axes={matching_axes} \
retired_temporary_vectors_per_disconnect=1 optimized_temporary_vectors_per_disconnect=0 \
retired_transition_writes_per_match=2 optimized_transition_writes_per_match=1 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "direct disconnect transition extension must reduce P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn axis_key(gamepad: u64, axis: GamepadAxis) -> (GamepadId, GamepadAxis) {
        (GamepadId(gamepad), axis)
    }

    fn transition(
        gamepad: u64,
        axis: GamepadAxis,
        previous_value: f32,
        value: f32,
    ) -> GamepadAxisTransition {
        GamepadAxisTransition {
            gamepad: GamepadId(gamepad),
            axis,
            previous_value,
            value,
        }
    }

    fn benchmark_axes(count: usize) -> BTreeMap<(GamepadId, GamepadAxis), f32> {
        (0..count)
            .map(|index| {
                (
                    axis_key((index & 1) as u64, GamepadAxis::Other((index / 2) as u16)),
                    if index & 1 == 0 { 0.5 } else { -0.5 },
                )
            })
            .collect()
    }

    fn retired_reset_gamepad_axes_for_focus_loss(
        axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
        transitions: &mut Vec<GamepadAxisTransition>,
    ) {
        for ((gamepad, axis), previous_value) in axes {
            if previous_value == 0.0 {
                continue;
            }
            if let Some(transition) = transitions
                .iter_mut()
                .find(|transition| transition.gamepad == gamepad && transition.axis == axis)
            {
                transition.value = 0.0;
            } else {
                transitions.push(GamepadAxisTransition {
                    gamepad,
                    axis,
                    previous_value,
                    value: 0.0,
                });
            }
        }
    }

    fn retired_append_disconnected_axis_transitions(
        axes: &BTreeMap<(GamepadId, GamepadAxis), f32>,
        transitions: &mut Vec<GamepadAxisTransition>,
        disconnected_gamepad: GamepadId,
    ) {
        let disconnected = axes
            .iter()
            .filter_map(|((gamepad, axis), value)| {
                (gamepad == &disconnected_gamepad && *value != 0.0).then_some(
                    GamepadAxisTransition {
                        gamepad: *gamepad,
                        axis: *axis,
                        previous_value: *value,
                        value: 0.0,
                    },
                )
            })
            .collect::<Vec<_>>();
        transitions.extend(disconnected);
    }

    fn measure_focus_reset(
        axes: &BTreeMap<(GamepadId, GamepadAxis), f32>,
        transitions: &[GamepadAxisTransition],
        reset: fn(BTreeMap<(GamepadId, GamepadAxis), f32>, &mut Vec<GamepadAxisTransition>),
    ) -> Duration {
        let inputs = (0..FOCUS_RESET_ITERATIONS)
            .map(|_| (axes.clone(), transitions.to_vec()))
            .collect::<Vec<_>>();
        let started = Instant::now();
        for (axes, mut transitions) in inputs {
            reset(axes, &mut transitions);
            black_box(transitions);
        }
        started.elapsed()
    }

    fn measure_disconnect_append(
        axes: &BTreeMap<(GamepadId, GamepadAxis), f32>,
        disconnected_gamepad: GamepadId,
        append: fn(
            &BTreeMap<(GamepadId, GamepadAxis), f32>,
            &mut Vec<GamepadAxisTransition>,
            GamepadId,
        ),
    ) -> Duration {
        let matching_axes = DISCONNECT_AXIS_COUNT / 2;
        let mut outputs = (0..DISCONNECT_ITERATIONS)
            .map(|_| Vec::with_capacity(matching_axes))
            .collect::<Vec<_>>();
        let started = Instant::now();
        for transitions in &mut outputs {
            append(axes, transitions, disconnected_gamepad);
        }
        black_box(outputs);
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
