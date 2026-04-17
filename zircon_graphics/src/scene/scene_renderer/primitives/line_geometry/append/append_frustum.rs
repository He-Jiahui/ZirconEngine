use zircon_math::{Transform, Vec3, Vec4};

use crate::scene::scene_renderer::primitives::LineVertex;

pub(crate) fn append_frustum(
    vertices: &mut Vec<LineVertex>,
    transform: Transform,
    fov_y_radians: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    color: Vec4,
) {
    let near_half_height = (fov_y_radians * 0.5).tan() * z_near;
    let near_half_width = near_half_height * aspect_ratio.max(0.001);
    let far_half_height = (fov_y_radians * 0.5).tan() * z_far;
    let far_half_width = far_half_height * aspect_ratio.max(0.001);
    let matrix = transform.matrix();
    let near = [
        Vec3::new(-near_half_width, near_half_height, -z_near),
        Vec3::new(near_half_width, near_half_height, -z_near),
        Vec3::new(near_half_width, -near_half_height, -z_near),
        Vec3::new(-near_half_width, -near_half_height, -z_near),
    ]
    .map(|point| matrix.transform_point3(point));
    let far = [
        Vec3::new(-far_half_width, far_half_height, -z_far),
        Vec3::new(far_half_width, far_half_height, -z_far),
        Vec3::new(far_half_width, -far_half_height, -z_far),
        Vec3::new(-far_half_width, -far_half_height, -z_far),
    ]
    .map(|point| matrix.transform_point3(point));
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
        vertices.push(LineVertex::new(near[a], color));
        vertices.push(LineVertex::new(near[b], color));
        vertices.push(LineVertex::new(far[a], color));
        vertices.push(LineVertex::new(far[b], color));
        vertices.push(LineVertex::new(near[a], color));
        vertices.push(LineVertex::new(far[a], color));
    }
}
