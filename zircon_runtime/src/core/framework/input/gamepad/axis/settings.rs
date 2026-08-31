use serde::{Deserialize, Serialize};

use super::super::value_scaling::linear_remapping;

pub const GAMEPAD_AXIS_DEADZONE_LOWER: f32 = -0.05;
pub const GAMEPAD_AXIS_DEADZONE_UPPER: f32 = 0.05;
pub const GAMEPAD_AXIS_LIVEZONE_LOWER: f32 = -1.0;
pub const GAMEPAD_AXIS_LIVEZONE_UPPER: f32 = 1.0;
pub const GAMEPAD_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisSettings {
    pub livezone_upperbound: f32,
    pub deadzone_upperbound: f32,
    pub deadzone_lowerbound: f32,
    pub livezone_lowerbound: f32,
    pub change_threshold: f32,
}

impl Default for GamepadAxisSettings {
    fn default() -> Self {
        Self {
            livezone_upperbound: GAMEPAD_AXIS_LIVEZONE_UPPER,
            deadzone_upperbound: GAMEPAD_AXIS_DEADZONE_UPPER,
            deadzone_lowerbound: GAMEPAD_AXIS_DEADZONE_LOWER,
            livezone_lowerbound: GAMEPAD_AXIS_LIVEZONE_LOWER,
            change_threshold: GAMEPAD_AXIS_CHANGE_THRESHOLD,
        }
    }
}

impl GamepadAxisSettings {
    pub const fn new(
        livezone_lowerbound: f32,
        deadzone_lowerbound: f32,
        deadzone_upperbound: f32,
        livezone_upperbound: f32,
        change_threshold: f32,
    ) -> Self {
        Self {
            livezone_upperbound,
            deadzone_upperbound,
            deadzone_lowerbound,
            livezone_lowerbound,
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
        let value = self.clamped_value(raw_value.clamp(-1.0, 1.0));
        if value == 0.0 {
            0.0
        } else if value >= self.livezone_upperbound {
            1.0
        } else if value <= self.livezone_lowerbound {
            -1.0
        } else if value >= self.deadzone_upperbound {
            linear_remapping(
                value,
                self.deadzone_upperbound,
                self.livezone_upperbound,
                0.0,
                1.0,
            )
        } else if value <= self.deadzone_lowerbound {
            linear_remapping(
                value,
                self.livezone_lowerbound,
                self.deadzone_lowerbound,
                -1.0,
                0.0,
            )
        } else {
            0.0
        }
    }

    fn clamped_value(self, value: f32) -> f32 {
        if self.deadzone_lowerbound <= value && value <= self.deadzone_upperbound {
            0.0
        } else if value >= self.livezone_upperbound {
            1.0
        } else if value <= self.livezone_lowerbound {
            -1.0
        } else {
            value
        }
    }
}
