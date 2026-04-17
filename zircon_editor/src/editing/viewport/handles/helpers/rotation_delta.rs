use crate::GizmoAxis;
use zircon_math::Quat;

use super::{global_axis, local_axis};

pub(in crate::editing::viewport::handles) fn local_rotation_delta(
    axis: GizmoAxis,
    angle_radians: f32,
) -> Quat {
    Quat::from_axis_angle(local_axis(axis), angle_radians)
}

pub(in crate::editing::viewport::handles) fn global_rotation_delta(
    axis: GizmoAxis,
    angle_radians: f32,
) -> Quat {
    Quat::from_axis_angle(global_axis(axis), angle_radians)
}
