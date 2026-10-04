use crate::core::framework::render::OverlayBillboardIcon;
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::IconVertex;

use super::super::size::icon_world_size;

/// 沿当前相机 right/up 构造面向视口的贴图图标；仅在图标 atlas 有可用绑定时由 gizmo 准备阶段调用。
pub(crate) fn build_icon_quad_vertices(
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
) -> [IconVertex; 6] {
    let half = icon_world_size(icon) * 0.5;
    let top_left = icon.position - right * half + up * half;
    let top_right = icon.position + right * half + up * half;
    let bottom_left = icon.position - right * half - up * half;
    let bottom_right = icon.position + right * half - up * half;
    [
        IconVertex::new(top_left, [0.0, 0.0], icon.tint),
        IconVertex::new(bottom_left, [0.0, 1.0], icon.tint),
        IconVertex::new(top_right, [1.0, 0.0], icon.tint),
        IconVertex::new(top_right, [1.0, 0.0], icon.tint),
        IconVertex::new(bottom_left, [0.0, 1.0], icon.tint),
        IconVertex::new(bottom_right, [1.0, 1.0], icon.tint),
    ]
}
