use crate::core::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCameraState {
    pub enabled: bool,
    pub target: Vec3,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            enabled: true,
            target: Vec3::ZERO,
        }
    }
}
