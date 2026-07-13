use crate::core::math::{Mat4, Vec3};

use super::PLANAR_PLANE_EPSILON;

/// Builds the affine reflection across a world-space plane.
pub fn planar_reflection_matrix(plane_point: Vec3, plane_normal: Vec3) -> Option<Mat4> {
    if !plane_point.is_finite() || !plane_normal.is_finite() {
        return None;
    }
    let normal_length_squared = plane_normal.length_squared();
    if normal_length_squared <= PLANAR_PLANE_EPSILON {
        return None;
    }
    let normal = plane_normal / normal_length_squared.sqrt();
    let reflect_vector = |vector: Vec3| vector - 2.0 * normal * normal.dot(vector);
    let reflected_x = reflect_vector(Vec3::X);
    let reflected_y = reflect_vector(Vec3::Y);
    let reflected_z = reflect_vector(Vec3::Z);
    let translation = plane_point - reflect_vector(plane_point);
    Some(Mat4::from_cols(
        reflected_x.extend(0.0),
        reflected_y.extend(0.0),
        reflected_z.extend(0.0),
        translation.extend(1.0),
    ))
}
