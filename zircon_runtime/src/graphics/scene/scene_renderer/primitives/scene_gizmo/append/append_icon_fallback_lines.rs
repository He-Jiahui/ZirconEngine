use crate::core::framework::render::{OverlayBillboardIcon, ViewportIconId};
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::icons::icon_world_size;
use super::append_camera_icon_fallback_lines::append_camera_icon_fallback_lines;
use super::append_directional_light_icon_fallback_lines::append_directional_light_icon_fallback_lines;

const CAMERA_ICON_FALLBACK_VERTEX_CAPACITY: usize = 12;
const DIRECTIONAL_LIGHT_ICON_FALLBACK_VERTEX_CAPACITY: usize = 8;

/// 与 append_icon_fallback_lines 使用同一图标分类，供上层精确预留回退顶点容量。
pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) fn icon_fallback_vertex_capacity(
    icon: &OverlayBillboardIcon,
) -> usize {
    match icon.id {
        ViewportIconId::Camera => CAMERA_ICON_FALLBACK_VERTEX_CAPACITY,
        ViewportIconId::DirectionalLight => DIRECTIONAL_LIGHT_ICON_FALLBACK_VERTEX_CAPACITY,
    }
}

/// 在图标 atlas 无对应资源时按图标语义补线框；调用方用 has 判定，Pending 上传仍视为有资源。
pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) fn append_icon_fallback_lines(
    vertices: &mut Vec<LineVertex>,
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
) {
    let size = icon_world_size(icon);
    match icon.id {
        ViewportIconId::Camera => {
            append_camera_icon_fallback_lines(vertices, icon, right, up, size)
        }
        ViewportIconId::DirectionalLight => {
            append_directional_light_icon_fallback_lines(vertices, icon, right, up, size)
        }
    }
}

#[cfg(test)]
#[path = "tests/append_icon_fallback_lines.rs"]
mod tests;
