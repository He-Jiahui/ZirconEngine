use crate::core::framework::render::RenderViewportHandle;
use crate::core::math::{UVec2, Vec2};

use super::PointerId;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 指针在某个渲染视口内的位置；射线构建要求 position 使用该视口的局部像素坐标。
pub struct PointerLocation {
    pub pointer: PointerId,
    pub viewport: RenderViewportHandle,
    pub position: Vec2,
}

impl PointerLocation {
    pub fn new(pointer: PointerId, viewport: RenderViewportHandle, position: Vec2) -> Self {
        Self {
            pointer,
            viewport,
            position,
        }
    }

    pub fn is_inside_viewport(self, viewport_size: UVec2) -> bool {
        let width = viewport_size.x as f32;
        let height = viewport_size.y as f32;
        self.position.x >= 0.0
            && self.position.y >= 0.0
            && self.position.x <= width
            && self.position.y <= height
    }
}
