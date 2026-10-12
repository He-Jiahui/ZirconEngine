use crate::core::editing::interactive_transform::{
    InteractiveTransformAxis, InteractiveTransformKind, InteractiveTransformSpace,
    InteractiveTransformSpec,
};
use crate::scene::viewport::{GizmoAxis, TransformSpace};

use super::handle_drag_session::HandleDragSession;

impl HandleDragSession {
    pub(crate) fn interactive_transform_spec(&self) -> InteractiveTransformSpec {
        let session = match self {
            Self::Move(session) | Self::Rotate(session) | Self::Scale(session) => session,
        };
        let kind = match self {
            Self::Move(_) => InteractiveTransformKind::Move,
            Self::Rotate(_) => InteractiveTransformKind::Rotate,
            Self::Scale(_) => InteractiveTransformKind::Scale,
        };
        let axis = match session.axis {
            GizmoAxis::X => InteractiveTransformAxis::X,
            GizmoAxis::Y => InteractiveTransformAxis::Y,
            GizmoAxis::Z => InteractiveTransformAxis::Z,
        };
        let space = match session.space {
            TransformSpace::Global => InteractiveTransformSpace::Global,
            TransformSpace::Local => InteractiveTransformSpace::Local,
        };
        InteractiveTransformSpec::new(kind, axis, space, session.snap_enabled)
    }
}
