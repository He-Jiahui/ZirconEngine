use crate::GizmoAxis;
use zircon_math::{Vec3, Vec4};
use zircon_scene::OverlayAxis;

use crate::editing::viewport::handles::{
    constants::{X_COLOR, Y_COLOR, Z_COLOR},
    handle_basis::HandleBasis,
};

pub(in crate::editing::viewport::handles) fn basis_axis(
    basis: &HandleBasis,
    axis: GizmoAxis,
) -> Vec3 {
    match axis {
        GizmoAxis::X => basis.x,
        GizmoAxis::Y => basis.y,
        GizmoAxis::Z => basis.z,
    }
}

pub(in crate::editing::viewport::handles) fn local_axis(axis: GizmoAxis) -> Vec3 {
    match axis {
        GizmoAxis::X => Vec3::X,
        GizmoAxis::Y => Vec3::Y,
        GizmoAxis::Z => -Vec3::Z,
    }
}

pub(in crate::editing::viewport::handles) fn global_axis(axis: GizmoAxis) -> Vec3 {
    local_axis(axis)
}

pub(in crate::editing::viewport::handles) fn axis_color(axis: OverlayAxis) -> Vec4 {
    match axis {
        OverlayAxis::X => X_COLOR,
        OverlayAxis::Y => Y_COLOR,
        OverlayAxis::Z => Z_COLOR,
    }
}
