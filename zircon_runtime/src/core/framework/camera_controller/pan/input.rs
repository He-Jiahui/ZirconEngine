use crate::core::math::{Real, UVec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 平面相机的一帧输入；拖拽像素按 viewport_size 与缩放状态换算为世界位移，键盘平移另按秒积分。
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
