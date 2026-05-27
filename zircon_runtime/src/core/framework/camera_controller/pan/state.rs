use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanCameraState {
    pub enabled: bool,
    pub zoom_factor: Real,
}

impl Default for PanCameraState {
    fn default() -> Self {
        Self {
            enabled: true,
            zoom_factor: 1.0,
        }
    }
}
