use crate::core::math::{Real, UVec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanCameraInput {
    pub delta_seconds: Real,
    pub pan_axis: Vec2,
    pub drag_delta: Vec2,
    pub zoom_delta: Real,
    pub rotate_axis: Real,
    pub viewport_size: UVec2,
}

impl Default for PanCameraInput {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            pan_axis: Vec2::ZERO,
            drag_delta: Vec2::ZERO,
            zoom_delta: 0.0,
            rotate_axis: 0.0,
            viewport_size: UVec2::ONE,
        }
    }
}
