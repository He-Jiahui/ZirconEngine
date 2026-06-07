use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::surface::UiNavigationEventKind;

use super::UiSurfaceInputState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceAnalogControlState {
    pub value: f32,
}

/// Repeat gate for one held analog navigation direction on a retained surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceAnalogNavigationState {
    pub kind: UiNavigationEventKind,
    pub last_navigation_time_micros: u64,
    pub repeats: u32,
}

impl UiSurfaceInputState {
    pub fn update_analog_control(&mut self, control: &str, value: f32) -> bool {
        const ANALOG_REPEAT_EPSILON: f32 = 0.001;

        let value = if value.is_finite() { value } else { 0.0 };
        match self.analog_controls.get_mut(control) {
            Some(state) if (state.value - value).abs() <= ANALOG_REPEAT_EPSILON => false,
            Some(state) => {
                state.value = value;
                true
            }
            None => {
                self.analog_controls
                    .insert(control.to_string(), UiSurfaceAnalogControlState { value });
                true
            }
        }
    }
}
