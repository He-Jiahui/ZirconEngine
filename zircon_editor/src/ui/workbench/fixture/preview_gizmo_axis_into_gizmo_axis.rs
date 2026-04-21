use crate::scene::viewport::GizmoAxis;

use super::PreviewGizmoAxis;

impl PreviewGizmoAxis {
    pub(crate) fn into_gizmo_axis(self) -> GizmoAxis {
        match self {
            Self::X => GizmoAxis::X,
            Self::Y => GizmoAxis::Y,
            Self::Z => GizmoAxis::Z,
        }
    }
}
