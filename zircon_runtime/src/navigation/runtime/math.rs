use crate::core::math::{Quat, Real, Vec3};

pub(super) fn rotation_from_direction(direction: Vec3) -> Quat {
    Quat::from_rotation_y(direction.x.atan2(-direction.z))
}

pub(super) fn distance_xz(left: Vec3, right: Vec3) -> Real {
    let delta = left - right;
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}
