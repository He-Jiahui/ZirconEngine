use crate::core::math::precision::{Mat4, Quat, Real, Vec3};
use crate::core::math::transform::Transform;
use glam::UVec2;

pub fn compose_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

pub fn transform_to_mat4(transform: Transform) -> Mat4 {
    compose_trs(transform.translation, transform.rotation, transform.scale)
}

pub fn affine_inverse(matrix: Mat4) -> Mat4 {
    matrix.inverse()
}

pub fn perspective(fov_y_radians: Real, aspect_ratio: Real, z_near: Real, z_far: Real) -> Mat4 {
    Mat4::perspective_rh(
        fov_y_radians,
        aspect_ratio.max(0.001),
        z_near.max(0.001),
        z_far,
    )
}

pub fn view_matrix(transform: Transform) -> Mat4 {
    affine_inverse(transform_to_mat4(transform))
}

pub fn clamp_viewport_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
