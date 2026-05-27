use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrbitCameraAction {
    None,
    Orbit { previous: Vec2, current: Vec2 },
    Pan { previous: Vec2, current: Vec2 },
    Zoom { delta: Real },
    Focus { target: Vec3 },
}

impl Default for OrbitCameraAction {
    fn default() -> Self {
        Self::None
    }
}
