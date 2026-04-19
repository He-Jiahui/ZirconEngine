use crate::core::framework::render::OverlayWireShape;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::line_geometry::{append_arrow_head, append_frustum};

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) fn append_wire_shape(
    vertices: &mut Vec<LineVertex>,
    shape: &OverlayWireShape,
) {
    match shape {
        OverlayWireShape::Frustum {
            transform,
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
            color,
        } => append_frustum(
            vertices,
            *transform,
            *fov_y_radians,
            *aspect_ratio,
            *z_near,
            *z_far,
            *color,
        ),
        OverlayWireShape::Arrow {
            origin,
            direction,
            length,
            color,
        } => {
            let end = *origin + direction.normalize_or_zero() * *length;
            vertices.push(LineVertex::new(*origin, *color));
            vertices.push(LineVertex::new(end, *color));
            append_arrow_head(vertices, *origin, end, *color);
        }
    }
}
