use crate::core::math::{Mat4, Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn append_bounding_box_vertices(
    vertices: &mut Vec<LineVertex>,
    min: Vec3,
    max: Vec3,
    transform: Mat4,
    color: Vec4,
) {
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ]
    .map(|corner| transform.transform_point3(corner));
    for (a, b) in [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        vertices.push(LineVertex::new(corners[a], color));
        vertices.push(LineVertex::new(corners[b], color));
    }
}
