use crate::core::math::{Mat4, Vec4};

use super::PLANAR_PLANE_EPSILON;

/// Replaces the near plane of a right-handed WGPU 0..1 projection.
pub fn planar_oblique_near_clip_projection(
    projection: Mat4,
    clip_plane_view: Vec4,
) -> Option<Mat4> {
    if !projection.is_finite() || !clip_plane_view.is_finite() {
        return None;
    }
    let normal_length = clip_plane_view.truncate().length();
    if normal_length <= PLANAR_PLANE_EPSILON {
        return None;
    }
    let plane = clip_plane_view / normal_length;
    let clip_corner = Vec4::new(far_corner_axis(plane.x), far_corner_axis(plane.y), 1.0, 1.0);
    let corner_view = projection.inverse() * clip_corner;
    if !corner_view.is_finite() {
        return None;
    }
    let denominator = plane.dot(corner_view);
    if !denominator.is_finite() || denominator.abs() <= PLANAR_PLANE_EPSILON {
        return None;
    }
    let near_row = (plane / denominator).to_array();
    let mut columns = projection.to_cols_array_2d();
    for column in 0..4 {
        columns[column][2] = near_row[column];
    }
    Some(Mat4::from_cols_array_2d(&columns))
}

const fn far_corner_axis(plane_axis: f32) -> f32 {
    if plane_axis < 0.0 {
        -1.0
    } else {
        1.0
    }
}
