use crate::core::math::{Vec3, Vec4};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn append_arrow_head(
    vertices: &mut Vec<LineVertex>,
    start: Vec3,
    end: Vec3,
    color: Vec4,
) {
    let forward = (end - start).normalize_or_zero();
    if forward.length_squared() <= f32::EPSILON {
        return;
    }
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let right = if right.length_squared() <= f32::EPSILON {
        forward.cross(Vec3::X).normalize_or_zero()
    } else {
        right
    };
    let head_length = (end - start).length() * 0.18;
    let head_width = head_length * 0.55;
    let base = end - forward * head_length;
    vertices.push(LineVertex::new(end, color));
    vertices.push(LineVertex::new(base + right * head_width, color));
    vertices.push(LineVertex::new(end, color));
    vertices.push(LineVertex::new(base - right * head_width, color));
}
