use crate::core::math::{Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

pub(crate) const CROSS_VERTEX_CAPACITY: usize = 4;

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

#[cfg(test)]
mod tests {
    use crate::core::math::{Vec3, Vec4};

    use super::{CROSS_VERTEX_CAPACITY, append_cross};

    #[test]
    fn cross_capacity_matches_output() {
        let mut vertices = Vec::new();
        append_cross(&mut vertices, Vec3::ZERO, 1.0, Vec4::ONE, Vec3::X, Vec3::Y);

        assert_eq!(vertices.len(), CROSS_VERTEX_CAPACITY);
    }
}
