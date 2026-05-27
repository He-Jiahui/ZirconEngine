use std::f32::consts::FRAC_PI_2;

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCameraSettings {
    pub sensitivity: Real,
    pub walk_speed: Real,
    pub run_speed: Real,
    pub scroll_factor: Real,
    pub friction: Real,
    pub pitch_min: Real,
    pub pitch_max: Real,
}

impl Default for FreeCameraSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.2,
            walk_speed: 5.0,
            run_speed: 15.0,
            scroll_factor: 0.04879016,
            friction: 40.0,
            pitch_min: -FRAC_PI_2,
            pitch_max: FRAC_PI_2,
        }
    }
}
