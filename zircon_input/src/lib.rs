//! Input module wired into the core runtime with a stable façade.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_manager::{
    InputButton, InputEvent, InputEventRecord, InputManager as InputManagerFacade,
    InputManagerHandle, InputSnapshot,
};
use zircon_module::{factory, qualified_name};

pub const INPUT_MODULE_NAME: &str = "InputModule";
pub const INPUT_DRIVER_NAME: &str = "InputModule.Driver.InputDriver";
pub const INPUT_MANAGER_NAME: &str = zircon_manager::INPUT_MANAGER_NAME;

#[derive(Clone, Debug, Default)]
pub struct InputConfig {
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct InputDriver;

#[derive(Debug, Default)]
pub struct DefaultInputManager {
    state: Mutex<InputState>,
}

#[derive(Debug, Default)]
struct InputState {
    cursor_position: [f32; 2],
    pressed_buttons: BTreeSet<InputButton>,
    wheel_accumulator: f32,
    events: Vec<InputEvent>,
    records: Vec<InputEventRecord>,
    next_sequence: u64,
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

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        INPUT_MODULE_NAME,
        "High-level input routing and action maps",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Driver, "InputDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(InputDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Manager, "InputManager"),
        StartupMode::Immediate,
        Vec::new(),
            factory(|_| {
                let manager = Arc::new(DefaultInputManager::default());
                Ok(Arc::new(InputManagerHandle::new(manager)) as ServiceObject)
            }),
        ))
}

#[cfg(test)]
mod tests;
