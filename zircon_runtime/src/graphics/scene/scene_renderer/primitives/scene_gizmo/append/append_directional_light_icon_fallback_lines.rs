use crate::core::framework::render::OverlayBillboardIcon;
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::line_geometry::append_cross;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo::append) fn append_directional_light_icon_fallback_lines(
    vertices: &mut Vec<LineVertex>,
    icon: &OverlayBillboardIcon,
    right: Vec3,
    up: Vec3,
    size: f32,
) {
    append_cross(vertices, icon.position, size, icon.tint, right, up);

    let diagonal = (right + up).normalize_or_zero();
    vertices.push(LineVertex::new(
        icon.position - diagonal * size * 0.7,
        icon.tint,
    ));
    vertices.push(LineVertex::new(
        icon.position + diagonal * size * 0.7,
        icon.tint,
    ));

    let diagonal = (right - up).normalize_or_zero();
    vertices.push(LineVertex::new(
        icon.position - diagonal * size * 0.7,
        icon.tint,
    ));
    vertices.push(LineVertex::new(
        icon.position + diagonal * size * 0.7,
        icon.tint,
    ));
}
