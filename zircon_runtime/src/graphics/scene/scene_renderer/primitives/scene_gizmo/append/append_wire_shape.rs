use crate::core::framework::render::OverlayWireShape;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

use super::super::super::line_geometry::{
    ARROW_HEAD_VERTEX_CAPACITY, append_arrow_head, append_frustum,
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
            let end = *origin + direction.normalize_or_zero() * *length;
            vertices.push(LineVertex::new(*origin, *color));
            vertices.push(LineVertex::new(end, *color));
            append_arrow_head(vertices, *origin, end, *color);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::OverlayWireShape;
    use crate::core::math::{Transform, Vec3, Vec4};

    use super::{append_wire_shape, wire_shape_vertex_capacity};

    #[test]
    fn scene_gizmo_line_capacity_matches_non_degenerate_wire_shapes() {
        let shapes = [
            OverlayWireShape::Frustum {
                transform: Transform::default(),
                fov_y_radians: 1.0,
                aspect_ratio: 1.5,
                z_near: 0.1,
                z_far: 10.0,
                color: Vec4::ONE,
            },
            OverlayWireShape::Arrow {
                origin: Vec3::ZERO,
                direction: Vec3::X,
                length: 1.0,
                color: Vec4::ONE,
            },
        ];

        for shape in shapes {
            let mut vertices = Vec::new();
            append_wire_shape(&mut vertices, &shape);

            assert_eq!(vertices.len(), wire_shape_vertex_capacity(&shape));
        }
    }
}
