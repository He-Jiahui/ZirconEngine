use serde::{Deserialize, Serialize};

pub const GAMEPAD_BUTTON_PRESS_THRESHOLD: f32 = 0.75;
pub const GAMEPAD_BUTTON_RELEASE_THRESHOLD: f32 = 0.65;
pub const GAMEPAD_BUTTON_AXIS_LOW: f32 = 0.05;
pub const GAMEPAD_BUTTON_AXIS_HIGH: f32 = 0.95;
pub const GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD: f32 = 0.01;
pub const GAMEPAD_AXIS_DEADZONE_LOWER: f32 = -0.05;
pub const GAMEPAD_AXIS_DEADZONE_UPPER: f32 = 0.05;
pub const GAMEPAD_AXIS_LIVEZONE_LOWER: f32 = -1.0;
pub const GAMEPAD_AXIS_LIVEZONE_UPPER: f32 = 1.0;
pub const GAMEPAD_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GamepadId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
    DPadX,
    DPadY,
    Other(u16),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadConnectionInfo {
    pub gamepad: GamepadId,
    pub connected: bool,
    pub name: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisState {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonValueState {
    pub gamepad: GamepadId,
    pub button: GamepadButton,
    pub value: f32,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadRumbleIntensity {
    pub strong_motor: f32,
    pub weak_motor: f32,
}

impl GamepadRumbleIntensity {
    pub const MAX: Self = Self {
        strong_motor: 1.0,
        weak_motor: 1.0,
    };
    pub const STRONG_MAX: Self = Self {
        strong_motor: 1.0,
        weak_motor: 0.0,
    };
    pub const WEAK_MAX: Self = Self {
        strong_motor: 0.0,
        weak_motor: 1.0,
    };

    pub const fn new(strong_motor: f32, weak_motor: f32) -> Self {
        Self {
            strong_motor,
            weak_motor,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            strong_motor: clamp_motor_intensity(self.strong_motor),
            weak_motor: clamp_motor_intensity(self.weak_motor),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamepadRumbleRequest {
    Add {
        gamepad: GamepadId,
        intensity: GamepadRumbleIntensity,
        duration_millis: u32,
    },
    Stop {
        gamepad: GamepadId,
    },
}

impl GamepadRumbleRequest {
    pub const fn add(
        gamepad: GamepadId,
        intensity: GamepadRumbleIntensity,
        duration_millis: u32,
    ) -> Self {
        Self::Add {
            gamepad,
            intensity,
            duration_millis,
        }
    }

    pub const fn stop(gamepad: GamepadId) -> Self {
        Self::Stop { gamepad }
    }

    pub const fn gamepad(self) -> GamepadId {
        match self {
            Self::Add { gamepad, .. } | Self::Stop { gamepad } => gamepad,
        }
    }
}

fn linear_remapping(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
    ((value - old_start) / (old_end - old_start)) * (new_end - new_start) + new_start
}

fn clamp_motor_intensity(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
