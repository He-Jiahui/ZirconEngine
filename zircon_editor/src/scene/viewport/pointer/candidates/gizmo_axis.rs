use crate::GizmoAxis;
use crate::scene::viewport::OverlayAxis;

pub(in crate::scene::viewport::pointer) fn gizmo_axis(axis: OverlayAxis) -> GizmoAxis {
    match axis {
        OverlayAxis::X => GizmoAxis::X,
        OverlayAxis::Y => GizmoAxis::Y,
        OverlayAxis::Z => GizmoAxis::Z,
    }
}
