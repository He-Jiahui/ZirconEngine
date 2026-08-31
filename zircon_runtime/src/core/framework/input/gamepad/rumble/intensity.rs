use serde::{Deserialize, Serialize};

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

fn clamp_motor_intensity(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
