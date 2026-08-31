use crate::core::framework::render::{OverlayBillboardIcon, ViewportIconId};
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::icons::icon_world_size;
use super::append_camera_icon_fallback_lines::append_camera_icon_fallback_lines;
use super::append_directional_light_icon_fallback_lines::append_directional_light_icon_fallback_lines;

const CAMERA_ICON_FALLBACK_VERTEX_CAPACITY: usize = 12;
const DIRECTIONAL_LIGHT_ICON_FALLBACK_VERTEX_CAPACITY: usize = 8;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) fn icon_fallback_vertex_capacity(
    icon: &OverlayBillboardIcon,
) -> usize {
    match icon.id {
        ViewportIconId::Camera => CAMERA_ICON_FALLBACK_VERTEX_CAPACITY,
        ViewportIconId::DirectionalLight => DIRECTIONAL_LIGHT_ICON_FALLBACK_VERTEX_CAPACITY,
    }
}

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
mod tests {
    use crate::core::framework::render::{OverlayBillboardIcon, ViewportIconId};
    use crate::core::math::{Vec3, Vec4};

    use super::{append_icon_fallback_lines, icon_fallback_vertex_capacity};

    #[test]
    fn scene_gizmo_line_capacity_matches_icon_fallbacks() {
        let icons = [
            OverlayBillboardIcon {
                id: ViewportIconId::Camera,
                position: Vec3::ZERO,
                tint: Vec4::ONE,
                size: 1.0,
            },
            OverlayBillboardIcon {
                id: ViewportIconId::DirectionalLight,
                position: Vec3::ONE,
                tint: Vec4::ONE,
                size: 1.0,
            },
        ];

        for icon in icons {
            let mut vertices = Vec::new();
            append_icon_fallback_lines(&mut vertices, &icon, Vec3::X, Vec3::Y);

            assert_eq!(vertices.len(), icon_fallback_vertex_capacity(&icon));
        }
    }
}
