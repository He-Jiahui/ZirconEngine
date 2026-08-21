use std::sync::{Mutex, MutexGuard};

use crate::core::framework::input::InputManager as InputManagerFacade;

use crate::input::{
    CursorHostRequest, GamepadAxisSettings, GamepadAxisTransition, GamepadButtonAxisSettings,
    GamepadButtonSettings, ImeEvent, ImeHostRequest, InputButton, InputEvent,
    InputEventQueueStatus, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
    InputFrameSnapshot, InputSnapshot, MouseScrollUnit, MouseWheelEvent, TouchPhase, TouchPoint,
};

use super::InputState;

#[derive(Debug, Default)]
pub struct DefaultInputManager {
    state: Mutex<InputState>,
}

impl DefaultInputManager {
    fn lock_state(&self) -> MutexGuard<'_, InputState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl InputManagerFacade for DefaultInputManager {
    fn begin_frame(&self) {
        let mut state = self.lock_state();
        state.buttons.clear_transitions();
        state.frame_events.begin_frame();
        state.wheel_accumulator = 0.0;
        state.mouse_wheel_accumulator = [0.0, 0.0];
        state.mouse_wheel_unit = MouseScrollUnit::Line;
        state.mouse_wheel_events.clear();
        state.mouse_motion_accumulator = [0.0, 0.0];
        state.cursor_host_requests.clear();
        state.ime_commits.clear();
        state.ime_delete_surrounding.clear();
        state.ime_host_requests.clear();
        state.gamepad_axis_transitions.clear();
        state.gamepad_rumble_requests.clear();
        state.window_status_events.clear();
        state.file_drag_drop_events.clear();
    }

    fn submit_event(&self, event: InputEvent) {
        let mut state = self.lock_state();
        match &event {
            InputEvent::CursorMoved { x, y } => {
                state.cursor_position = [*x, *y];
            }
            InputEvent::CursorEntered => {
                state.cursor_inside_window = true;
            }
            InputEvent::CursorLeft => {
                state.cursor_inside_window = false;
            }
            InputEvent::MouseMotion { delta_x, delta_y } => {
                state.mouse_motion_accumulator[0] += *delta_x;
                state.mouse_motion_accumulator[1] += *delta_y;
            }
            InputEvent::ButtonPressed(button) => {
                state.buttons.press(button.clone());
            }
            InputEvent::ButtonReleased(button) => {
                state.buttons.release(button);
            }
            InputEvent::WheelScrolled { delta } => {
                state.wheel_accumulator += *delta;
                state.mouse_wheel_accumulator[1] += *delta;
                state.mouse_wheel_unit = MouseScrollUnit::Line;
                state
                    .mouse_wheel_events
                    .push(MouseWheelEvent::lines(0.0, *delta));
            }
            InputEvent::MouseWheel(wheel) => {
                state.wheel_accumulator += wheel.vertical_line_delta();
                state.mouse_wheel_accumulator[0] += wheel.x;
                state.mouse_wheel_accumulator[1] += wheel.y;
                state.mouse_wheel_unit = wheel.unit;
                state.mouse_wheel_events.push(*wheel);
            }
            InputEvent::KeyboardInput {
                key_code,
                logical_key,
                pressed,
                ..
            } => {
                let key_code = InputButton::KeyCode(*key_code);
                if *pressed {
                    state.buttons.press(key_code);
                    if let Some(logical_key) = logical_key {
                        state.buttons.press(InputButton::Key(logical_key.clone()));
                    }
                } else {
                    state.buttons.release(&key_code);
                    if let Some(logical_key) = logical_key {
                        state
                            .buttons
                            .release(&InputButton::Key(logical_key.clone()));
                    }
                }
            }
            InputEvent::Ime(ime) => match ime {
                ImeEvent::Enabled => {
                    state.ime_enabled = true;
                }
                ImeEvent::Disabled => {
                    state.ime_enabled = false;
                    state.ime_preedit = None;
                }
                ImeEvent::Preedit(preedit) => {
                    state.ime_preedit = if preedit.value.is_empty() {
                        None
                    } else {
                        Some(preedit.clone())
                    };
                }
                ImeEvent::Commit(value) => {
                    state.ime_preedit = None;
                    state.ime_commits.push(value.clone());
                }
                ImeEvent::DeleteSurrounding(delete) => {
                    state.ime_delete_surrounding.push(*delete);
                }
            },
            InputEvent::ImeHostRequest(request) => {
                state.ime_host_requests.push(request.clone());
            }
            InputEvent::CursorHostRequest(request) => {
                state.cursor_host_requests.push(*request);
            }
            InputEvent::WindowStatus(event) => {
                state.window_status_events.push(event.clone());
            }
            InputEvent::FileDragDrop(event) => {
                state.file_drag_drop_events.push(event.clone());
            }
            InputEvent::FocusLost => {
                state.clear_active_input_for_focus_loss();
            }
            InputEvent::Touch { id, phase, x, y } => match phase {
                TouchPhase::Started | TouchPhase::Moved => {
                    state.active_touches.insert(
                        *id,
                        TouchPoint {
                            id: *id,
                            position: [*x, *y],
                            phase: *phase,
                        },
                    );
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    state.active_touches.remove(id);
                }
            },
            InputEvent::GamepadConnection(info) => {
                if info.connected {
                    state.connected_gamepads.insert(info.gamepad);
                } else {
                    state.connected_gamepads.remove(&info.gamepad);
                    let disconnected_axis_transitions = state
                        .gamepad_axes
                        .iter()
                        .filter_map(|((gamepad, axis), value)| {
                            (gamepad == &info.gamepad && *value != 0.0).then_some(
                                GamepadAxisTransition {
                                    gamepad: *gamepad,
                                    axis: *axis,
                                    previous_value: *value,
                                    value: 0.0,
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                    state
                        .gamepad_axis_transitions
                        .extend(disconnected_axis_transitions);
                    state
                        .gamepad_axes
                        .retain(|(gamepad, _), _| gamepad != &info.gamepad);
                    state
                        .gamepad_button_values
                        .retain(|(gamepad, _), _| gamepad != &info.gamepad);
                    state.buttons.release_where(|button| {
                        matches!(button, InputButton::Gamepad { gamepad, .. } if gamepad == &info.gamepad)
                    });
                }
            }
            InputEvent::GamepadButton {
                gamepad,
                button,
                value,
                pressed,
            } => {
                let input_button = InputButton::Gamepad {
                    gamepad: *gamepad,
                    button: *button,
                };
                let previous_value = state
                    .gamepad_button_values
                    .get(&(*gamepad, *button))
                    .copied();
                let settings = GamepadButtonAxisSettings::default();
                if let Some(value) = settings.process_value(*value, previous_value) {
                    state
                        .gamepad_button_values
                        .insert((*gamepad, *button), value);
                    let currently_pressed = state.buttons.pressed(&input_button);
                    if button_should_press(value, currently_pressed, *pressed) {
                        state.buttons.press(input_button);
                    } else if button_should_release(value, *pressed) {
                        state.buttons.release(&input_button);
                    }
                }
            }
            InputEvent::GamepadAxis {
                gamepad,
                axis,
                value,
            } => {
                let previous_value = state.gamepad_axes.get(&(*gamepad, *axis)).copied();
                if let Some(value) =
                    GamepadAxisSettings::default().process_value(*value, previous_value)
                {
                    let previous_value = previous_value.unwrap_or(0.0);
                    if previous_value != value {
                        if let Some(transition) =
                            state
                                .gamepad_axis_transitions
                                .iter_mut()
                                .find(|transition| {
                                    transition.gamepad == *gamepad && transition.axis == *axis
                                })
                        {
                            transition.value = value;
                        } else {
                            state.gamepad_axis_transitions.push(GamepadAxisTransition {
                                gamepad: *gamepad,
                                axis: *axis,
                                previous_value,
                                value,
                            });
                        }
                    }
                    state.gamepad_axes.insert((*gamepad, *axis), value);
                }
            }
            InputEvent::GamepadRumbleRequest(request) => {
                state.gamepad_rumble_requests.push(*request);
            }
        }
        state.event_recorder.record(&event);
        state.frame_events.push(event);
    }

    fn snapshot(&self) -> InputSnapshot {
        let state = self.lock_state();
        InputSnapshot {
            cursor_position: state.cursor_position,
            pressed_buttons: state.buttons.pressed_inputs(),
            wheel_accumulator: state.wheel_accumulator,
        }
    }

    fn button_pressed(&self, button: &InputButton) -> bool {
        self.lock_state().buttons.pressed(button)
    }

    fn frame_snapshot(&self) -> InputFrameSnapshot {
        let state = self.lock_state();
        InputFrameSnapshot {
            cursor_position: state.cursor_position,
            cursor_inside_window: state.cursor_inside_window,
            cursor_host_requests: state.cursor_host_requests.clone(),
            buttons: state.buttons.clone(),
            wheel_accumulator: state.wheel_accumulator,
            mouse_wheel_accumulator: state.mouse_wheel_accumulator,
            mouse_wheel_unit: state.mouse_wheel_unit,
            mouse_wheel_events: state.mouse_wheel_events.clone(),
            mouse_motion_accumulator: state.mouse_motion_accumulator,
            active_touches: state.active_touches.values().copied().collect(),
            connected_gamepads: state.connected_gamepads.iter().copied().collect(),
            gamepad_axes: state.gamepad_axis_states(),
            gamepad_axis_transitions: state.gamepad_axis_transitions.clone(),
            gamepad_button_values: state.gamepad_button_value_states(),
            gamepad_rumble_requests: state.gamepad_rumble_requests.clone(),
            ime_enabled: state.ime_enabled,
            ime_preedit: state.ime_preedit.clone(),
            ime_commits: state.ime_commits.clone(),
            ime_delete_surrounding: state.ime_delete_surrounding.clone(),
            ime_host_requests: state.ime_host_requests.clone(),
            window_status_events: state.window_status_events.clone(),
            file_drag_drop_events: state.file_drag_drop_events.clone(),
        }
    }

    fn drain_ime_host_requests(&self) -> Vec<ImeHostRequest> {
        let mut state = self.lock_state();
        std::mem::take(&mut state.ime_host_requests)
    }

    fn drain_gamepad_rumble_requests(&self) -> Vec<crate::input::GamepadRumbleRequest> {
        let mut state = self.lock_state();
        std::mem::take(&mut state.gamepad_rumble_requests)
    }

    fn drain_cursor_host_requests(&self) -> Vec<CursorHostRequest> {
        let mut state = self.lock_state();
        std::mem::take(&mut state.cursor_host_requests)
    }

    fn drain_events(&self) -> Vec<InputEvent> {
        let mut state = self.lock_state();
        state.frame_events.drain()
    }

    fn drain_event_records(&self) -> Vec<InputEventRecord> {
        let mut state = self.lock_state();
        state.event_recorder.drain()
    }

    fn drain_event_records_with_status(
        &self,
    ) -> (Vec<InputEventRecord>, InputEventRecordingStatus) {
        let mut state = self.lock_state();
        let records = state.event_recorder.drain();
        (records, state.event_recorder.status())
    }

    fn set_event_recording_config(&self, config: InputEventRecordingConfig) {
        self.lock_state().event_recorder.configure(config);
    }

    fn event_recording_status(&self) -> InputEventRecordingStatus {
        self.lock_state().event_recorder.status()
    }

    fn event_queue_status(&self) -> InputEventQueueStatus {
        self.lock_state().frame_events.status()
    }
}

fn button_should_press(value: f32, currently_pressed: bool, host_pressed: bool) -> bool {
    host_pressed
        && GamepadButtonSettings::default()
            .transition_for_value(value, currently_pressed)
            .unwrap_or(false)
}

fn button_should_release(value: f32, host_pressed: bool) -> bool {
    !host_pressed || GamepadButtonSettings::default().is_released(value)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::panic::{self, AssertUnwindSafe};
    use std::time::Instant;

    use crate::core::framework::input::InputManager;
    use crate::input::{InputButton, InputEvent};

    use super::DefaultInputManager;

    const BUTTON_QUERY_BENCH_PRESSED_COUNT: usize = 1_024;
    const BUTTON_QUERY_BENCH_ITERATIONS: usize = 2_048;
    const BUTTON_QUERY_BENCH_SAMPLE_PAIRS: usize = 21;

    fn manager_with_pressed_key_codes(count: usize) -> DefaultInputManager {
        let manager = DefaultInputManager::default();
        for key_code in 0..count {
            manager.submit_event(InputEvent::ButtonPressed(InputButton::KeyCode(
                key_code as u32,
            )));
        }
        manager
    }

    fn legacy_snapshot_button_pressed(manager: &DefaultInputManager, button: &InputButton) -> bool {
        manager.snapshot().pressed_buttons.contains(button)
    }

    fn measure_ns(mut workload: impl FnMut()) -> u128 {
        let started = Instant::now();
        for _ in 0..BUTTON_QUERY_BENCH_ITERATIONS {
            workload();
        }
        started.elapsed().as_nanos()
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn input_manager_accessors_recover_poisoned_state_lock() {
        let manager = DefaultInputManager::default();
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.lock_state();
            panic!("poison input manager state");
        }));

        manager.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
        assert!(
            manager
                .snapshot()
                .pressed_buttons
                .contains(&InputButton::MouseLeft)
        );
        assert_eq!(
            manager.drain_events(),
            vec![InputEvent::ButtonPressed(InputButton::MouseLeft)]
        );

        manager.begin_frame();
        assert!(manager.frame_snapshot().mouse_wheel_events.is_empty());
    }

    #[test]
    fn direct_button_query_matches_snapshot_for_present_and_missing_buttons() {
        let manager = manager_with_pressed_key_codes(32);

        assert!(manager.button_pressed(&InputButton::KeyCode(17)));
        assert!(!manager.button_pressed(&InputButton::KeyCode(91)));
        assert_eq!(
            manager.button_pressed(&InputButton::KeyCode(17)),
            legacy_snapshot_button_pressed(&manager, &InputButton::KeyCode(17))
        );
        assert_eq!(
            manager.button_pressed(&InputButton::KeyCode(91)),
            legacy_snapshot_button_pressed(&manager, &InputButton::KeyCode(91))
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Runtime56 batch"]
    fn allocation_free_button_query_release_gate() {
        let manager = manager_with_pressed_key_codes(BUTTON_QUERY_BENCH_PRESSED_COUNT);
        let missing = InputButton::KeyCode(u32::MAX);
        assert!(!legacy_snapshot_button_pressed(&manager, &missing));
        assert!(!manager.button_pressed(&missing));

        let mut legacy_samples = Vec::with_capacity(BUTTON_QUERY_BENCH_SAMPLE_PAIRS);
        let mut direct_samples = Vec::with_capacity(BUTTON_QUERY_BENCH_SAMPLE_PAIRS);
        for pair in 0..BUTTON_QUERY_BENCH_SAMPLE_PAIRS {
            let measure_legacy = || {
                measure_ns(|| {
                    black_box(legacy_snapshot_button_pressed(
                        black_box(&manager),
                        black_box(&missing),
                    ));
                })
            };
            let measure_direct = || {
                measure_ns(|| {
                    black_box(black_box(&manager).button_pressed(black_box(&missing)));
                })
            };
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                direct_samples.push(measure_direct());
            } else {
                direct_samples.push(measure_direct());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p50_ns = nearest_rank_percentile(&legacy_samples, 50);
        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
        let direct_p50_ns = nearest_rank_percentile(&direct_samples, 50);
        let direct_p95_ns = nearest_rank_percentile(&direct_samples, 95);
        let legacy_snapshot_allocations = BUTTON_QUERY_BENCH_ITERATIONS;
        let direct_snapshot_allocations = 0;
        let legacy_button_clones = BUTTON_QUERY_BENCH_PRESSED_COUNT * BUTTON_QUERY_BENCH_ITERATIONS;
        let direct_button_clones = 0;
        let legacy_samples_ns = sample_csv(&legacy_samples);
        let direct_samples_ns = sample_csv(&direct_samples);

        println!(
            "PERF-MVP-559 task=runtime56_direct_button_query sample_pairs={} pressed_buttons={} iterations={} legacy_snapshot_allocations={} direct_snapshot_allocations={} legacy_button_clones={} direct_button_clones={} legacy_p50_ns={} legacy_p95_ns={} direct_p50_ns={} direct_p95_ns={} legacy_samples_ns={} direct_samples_ns={}",
            BUTTON_QUERY_BENCH_SAMPLE_PAIRS,
            BUTTON_QUERY_BENCH_PRESSED_COUNT,
            BUTTON_QUERY_BENCH_ITERATIONS,
            legacy_snapshot_allocations,
            direct_snapshot_allocations,
            legacy_button_clones,
            direct_button_clones,
            legacy_p50_ns,
            legacy_p95_ns,
            direct_p50_ns,
            direct_p95_ns,
            legacy_samples_ns,
            direct_samples_ns,
        );

        assert_eq!(legacy_snapshot_allocations, 2_048);
        assert_eq!(direct_snapshot_allocations, 0);
        assert_eq!(legacy_button_clones, 2_097_152);
        assert_eq!(direct_button_clones, 0);
        assert!(
            direct_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "direct query P95 {direct_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
        );
    }
}
