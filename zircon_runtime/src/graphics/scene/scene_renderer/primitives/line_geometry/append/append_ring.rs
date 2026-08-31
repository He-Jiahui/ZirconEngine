use crate::core::math::{Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

const RING_SEGMENTS: usize = 48;
pub(crate) const RING_VERTEX_CAPACITY: usize = RING_SEGMENTS * 2;

pub(crate) fn append_ring(
    vertices: &mut Vec<LineVertex>,
    center: Vec3,
    normal: Vec3,
    radius: f32,
    color: Vec4,
) {
    let normal = normal.normalize_or_zero();
    if normal.length_squared() <= f32::EPSILON {
        return;
    }
    let tangent = if normal.cross(Vec3::Y).length_squared() > f32::EPSILON {
        normal.cross(Vec3::Y).normalize_or_zero()
    } else {
        normal.cross(Vec3::X).normalize_or_zero()
    };
    let bitangent = normal.cross(tangent).normalize_or_zero();
    let mut previous = center + tangent * radius;
    for step in 1..=RING_SEGMENTS {
        let angle = std::f32::consts::TAU * step as f32 / RING_SEGMENTS as f32;
        let next = center + (tangent * angle.cos() + bitangent * angle.sin()) * radius;
        vertices.push(LineVertex::new(previous, color));
        vertices.push(LineVertex::new(next, color));
        previous = next;
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::{Vec3, Vec4};

    use super::{RING_VERTEX_CAPACITY, append_ring};

    #[test]
    fn ring_capacity_matches_non_degenerate_output() {
        let mut vertices = Vec::new();
        append_ring(&mut vertices, Vec3::ZERO, Vec3::Z, 1.0, Vec4::ONE);

        assert_eq!(vertices.len(), RING_VERTEX_CAPACITY);
    }
}
