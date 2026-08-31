use serde::{Deserialize, Serialize};

pub const GAMEPAD_BUTTON_PRESS_THRESHOLD: f32 = 0.75;
pub const GAMEPAD_BUTTON_RELEASE_THRESHOLD: f32 = 0.65;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonSettings {
    pub press_threshold: f32,
    pub release_threshold: f32,
}

impl Default for GamepadButtonSettings {
    fn default() -> Self {
        Self {
            press_threshold: GAMEPAD_BUTTON_PRESS_THRESHOLD,
            release_threshold: GAMEPAD_BUTTON_RELEASE_THRESHOLD,
        }
    }
}

impl GamepadButtonSettings {
    pub const fn new(press_threshold: f32, release_threshold: f32) -> Self {
        Self {
            press_threshold,
            release_threshold,
        }
    }

    pub fn is_pressed(self, value: f32) -> bool {
        value >= self.press_threshold
    }

    pub fn is_released(self, value: f32) -> bool {
        value <= self.release_threshold
    }

    pub fn transition_for_value(self, value: f32, currently_pressed: bool) -> Option<bool> {
        if currently_pressed {
            self.is_released(value).then_some(false)
        } else {
            self.is_pressed(value).then_some(true)
        }
    }
}
