use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCameraSettings {
    pub orbit_sensitivity: Real,
    pub pan_sensitivity: Real,
    pub zoom_fraction: Real,
    pub min_distance: Real,
    pub pitch_min: Real,
    pub pitch_max: Real,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.01,
            pan_sensitivity: 0.0015,
            zoom_fraction: 0.15,
            min_distance: 0.25,
            pitch_min: -1.4,
            pitch_max: 1.4,
        }
    }
}
