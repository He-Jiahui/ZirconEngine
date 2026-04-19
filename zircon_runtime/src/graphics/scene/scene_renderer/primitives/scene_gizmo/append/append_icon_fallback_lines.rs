use crate::core::framework::render::{OverlayBillboardIcon, ViewportIconId};
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::icons::icon_world_size;
use super::append_camera_icon_fallback_lines::append_camera_icon_fallback_lines;
use super::append_directional_light_icon_fallback_lines::append_directional_light_icon_fallback_lines;

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
