use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCameraState {
    pub enabled: bool,
    pub pitch: Real,
    pub yaw: Real,
    pub speed_multiplier: Real,
    pub velocity: Vec3,
}

impl Default for FreeCameraState {
    fn default() -> Self {
        Self {
            enabled: true,
            pitch: 0.0,
            yaw: 0.0,
            speed_multiplier: 1.0,
            velocity: Vec3::ZERO,
        }
    }
}
