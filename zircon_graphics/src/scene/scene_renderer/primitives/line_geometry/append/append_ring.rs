use zircon_math::{Vec3, Vec4};

use crate::scene::scene_renderer::primitives::LineVertex;

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
    const SEGMENTS: usize = 48;
    let mut previous = center + tangent * radius;
    for step in 1..=SEGMENTS {
        let angle = std::f32::consts::TAU * step as f32 / SEGMENTS as f32;
        let next = center + (tangent * angle.cos() + bitangent * angle.sin()) * radius;
        vertices.push(LineVertex::new(previous, color));
        vertices.push(LineVertex::new(next, color));
        previous = next;
    }
}
