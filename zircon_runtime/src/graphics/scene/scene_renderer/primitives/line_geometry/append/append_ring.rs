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
    // 先选不与法线平行的参考轴，再用两次叉积构成圆环平面基底。
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
#[path = "tests/append_ring.rs"]
mod tests;
