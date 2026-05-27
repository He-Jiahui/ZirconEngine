use std::f32::consts::PI;

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanCameraSettings {
    pub pan_speed: Real,
    pub zoom_speed: Real,
    pub min_zoom: Real,
    pub max_zoom: Real,
    pub rotation_speed: Real,
    pub drag_pan_speed: Real,
}

impl Default for PanCameraSettings {
    fn default() -> Self {
        Self {
            pan_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.1,
            max_zoom: 5.0,
            rotation_speed: PI,
            drag_pan_speed: 1.0,
        }
    }
}
