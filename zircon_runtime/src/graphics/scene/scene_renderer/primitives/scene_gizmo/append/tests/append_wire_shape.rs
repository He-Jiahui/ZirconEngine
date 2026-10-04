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
