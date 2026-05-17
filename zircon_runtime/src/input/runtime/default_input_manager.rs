use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::input::InputManager as InputManagerFacade;

use crate::input::{
    ImeEvent, ImeHostRequest, InputButton, InputEvent, InputEventRecord, InputFrameSnapshot,
    InputSnapshot, MouseScrollUnit, MouseWheelEvent, TouchPhase, TouchPoint,
};

use super::InputState;

#[derive(Debug, Default)]
pub struct DefaultInputManager {
    state: Mutex<InputState>,
}

impl InputManagerFacade for DefaultInputManager {
    fn begin_frame(&self) {
        let mut state = self.state.lock().unwrap();
        state.buttons.clear_transitions();
        state.wheel_accumulator = 0.0;
        state.mouse_wheel_accumulator = [0.0, 0.0];
        state.mouse_wheel_unit = MouseScrollUnit::Line;
        state.mouse_wheel_events.clear();
        state.mouse_motion_accumulator = [0.0, 0.0];
        state.ime_commits.clear();
        state.ime_delete_surrounding.clear();
        state.ime_host_requests.clear();
        state.window_status_events.clear();
        state.file_drag_drop_events.clear();
    }

    fn submit_event(&self, event: InputEvent) {
        let mut state = self.state.lock().unwrap();
        state.next_sequence += 1;
        let sequence = state.next_sequence;
        let timestamp_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
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
                state.wheel_accumulator += wheel.legacy_vertical_delta();
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
            InputEvent::WindowStatus(event) => {
                state.window_status_events.push(event.clone());
            }
            InputEvent::FileDragDrop(event) => {
                state.file_drag_drop_events.push(event.clone());
            }
            InputEvent::KeyboardFocusLost => {
                state.buttons.release_where(|button| {
                    matches!(button, InputButton::Key(_) | InputButton::KeyCode(_))
                });
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
                    state
                        .gamepad_axes
                        .retain(|(gamepad, _), _| gamepad != &info.gamepad);
                    state.buttons.release_where(|button| {
                        matches!(button, InputButton::Gamepad { gamepad, .. } if gamepad == &info.gamepad)
                    });
                }
            }
            InputEvent::GamepadButton {
                gamepad,
                button,
                pressed,
                ..
            } => {
                let button = InputButton::Gamepad {
                    gamepad: *gamepad,
                    button: *button,
                };
                if *pressed {
                    state.buttons.press(button);
                } else {
                    state.buttons.release(&button);
                }
            }
            InputEvent::GamepadAxis {
                gamepad,
                axis,
                value,
            } => {
                state.gamepad_axes.insert((*gamepad, *axis), *value);
            }
        }
        state.events.push(event.clone());
        state.records.push(InputEventRecord {
            sequence,
            timestamp_millis,
            event,
        });
    }

    fn snapshot(&self) -> InputSnapshot {
        let state = self.state.lock().unwrap();
        InputSnapshot {
            cursor_position: state.cursor_position,
            pressed_buttons: state.buttons.pressed_inputs(),
            wheel_accumulator: state.wheel_accumulator,
        }
    }

    fn frame_snapshot(&self) -> InputFrameSnapshot {
        let state = self.state.lock().unwrap();
        InputFrameSnapshot {
            cursor_position: state.cursor_position,
            cursor_inside_window: state.cursor_inside_window,
            buttons: state.buttons.clone(),
            wheel_accumulator: state.wheel_accumulator,
            mouse_wheel_accumulator: state.mouse_wheel_accumulator,
            mouse_wheel_unit: state.mouse_wheel_unit,
            mouse_wheel_events: state.mouse_wheel_events.clone(),
            mouse_motion_accumulator: state.mouse_motion_accumulator,
            active_touches: state.active_touches.values().copied().collect(),
            connected_gamepads: state.connected_gamepads.iter().copied().collect(),
            gamepad_axes: state.gamepad_axis_states(),
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
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.ime_host_requests)
    }

    fn drain_events(&self) -> Vec<InputEvent> {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.events)
    }

    fn drain_event_records(&self) -> Vec<InputEventRecord> {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.records)
    }
}
