use crate::core::math::Vec3;

pub(super) fn mesh_bounds(positions: &[Vec3]) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for position in positions {
        min = min.min(*position);
        max = max.max(*position);
    }
    if !min.is_finite() || !max.is_finite() {
        (Vec3::ZERO, Vec3::ZERO)
    } else {
        (min, max)
    }
}
