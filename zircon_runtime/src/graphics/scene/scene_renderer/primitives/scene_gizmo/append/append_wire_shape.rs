use crate::core::framework::render::OverlayWireShape;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::line_geometry::{
    append_arrow_head, append_frustum, ARROW_HEAD_VERTEX_CAPACITY,
};

const ARROW_LINE_VERTEX_CAPACITY: usize = 2;
const FRUSTUM_VERTEX_CAPACITY: usize = 24;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) fn wire_shape_vertex_capacity(
    shape: &OverlayWireShape,
) -> usize {
    match shape {
        OverlayWireShape::Frustum { .. } => FRUSTUM_VERTEX_CAPACITY,
        OverlayWireShape::Arrow { .. } => {
            ARROW_LINE_VERTEX_CAPACITY.saturating_add(ARROW_HEAD_VERTEX_CAPACITY)
        }
    }
}

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
            // 请求长度独立于方向向量的模长，先归一化方向再应用 length。
            let end = *origin + direction.normalize_or_zero() * *length;
            vertices.push(LineVertex::new(*origin, *color));
            vertices.push(LineVertex::new(end, *color));
            append_arrow_head(vertices, *origin, end, *color);
        }
    }
}

#[cfg(test)]
#[path = "tests/append_wire_shape.rs"]
mod tests;
