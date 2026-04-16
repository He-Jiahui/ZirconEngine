use zircon_graphics::GizmoAxis;
use zircon_scene::OverlayAxis;

pub(in crate::editing::viewport::pointer) fn gizmo_axis(axis: OverlayAxis) -> GizmoAxis {
    match axis {
        OverlayAxis::X => GizmoAxis::X,
        OverlayAxis::Y => GizmoAxis::Y,
        OverlayAxis::Z => GizmoAxis::Z,
    }
}
