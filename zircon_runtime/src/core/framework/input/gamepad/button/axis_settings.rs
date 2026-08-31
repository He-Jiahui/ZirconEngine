use serde::{Deserialize, Serialize};

use super::super::value_scaling::linear_remapping;

pub const GAMEPAD_BUTTON_AXIS_LOW: f32 = 0.05;
pub const GAMEPAD_BUTTON_AXIS_HIGH: f32 = 0.95;
pub const GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonAxisSettings {
    pub low: f32,
    pub high: f32,
    pub change_threshold: f32,
}

impl Default for GamepadButtonAxisSettings {
    fn default() -> Self {
        Self {
            low: GAMEPAD_BUTTON_AXIS_LOW,
            high: GAMEPAD_BUTTON_AXIS_HIGH,
            change_threshold: GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD,
        }
    }
}

impl GamepadButtonAxisSettings {
    pub const fn new(low: f32, high: f32, change_threshold: f32) -> Self {
        Self {
            low,
            high,
            change_threshold,
        }
    }

    pub fn process_value(self, raw_value: f32, previous_value: Option<f32>) -> Option<f32> {
        if !raw_value.is_finite() {
            return None;
        }
        let value = self.scaled_value(raw_value);
        if previous_value
            .map(|previous| (value - previous).abs() >= self.change_threshold)
            .unwrap_or(true)
        {
            Some(value)
        } else {
            None
        }
    }

    pub fn scaled_value(self, raw_value: f32) -> f32 {
        let value = raw_value.clamp(0.0, 1.0);
        if value <= self.low {
            0.0
        } else if value >= self.high {
            1.0
        } else {
            linear_remapping(value, self.low, self.high, 0.0, 1.0)
        }
    }
}
