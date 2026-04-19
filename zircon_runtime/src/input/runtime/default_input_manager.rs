use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::input::InputManager as InputManagerFacade;

use crate::input::{InputEvent, InputEventRecord, InputSnapshot};

use super::InputState;

#[derive(Debug, Default)]
pub struct DefaultInputManager {
    state: Mutex<InputState>,
}

impl InputManagerFacade for DefaultInputManager {
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
            InputEvent::ButtonPressed(button) => {
                state.pressed_buttons.insert(button.clone());
            }
            InputEvent::ButtonReleased(button) => {
                state.pressed_buttons.remove(button);
            }
            InputEvent::WheelScrolled { delta } => {
                state.wheel_accumulator += *delta;
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
            pressed_buttons: state.pressed_buttons.iter().cloned().collect(),
            wheel_accumulator: state.wheel_accumulator,
        }
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
