use crate::core::math::{Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn append_cross(
    vertices: &mut Vec<LineVertex>,
    position: Vec3,
    size: f32,
    color: Vec4,
    right: Vec3,
    up: Vec3,
) {
    vertices.push(LineVertex::new(position - right * size, color));
    vertices.push(LineVertex::new(position + right * size, color));
    vertices.push(LineVertex::new(position - up * size, color));
    vertices.push(LineVertex::new(position + up * size, color));
}
