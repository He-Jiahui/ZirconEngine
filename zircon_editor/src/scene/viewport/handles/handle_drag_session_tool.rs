use crate::scene::viewport::TransformHandleKind;

use super::handle_drag_session::HandleDragSession;

impl HandleDragSession {
    pub(in crate::scene::viewport::handles) fn kind(&self) -> TransformHandleKind {
        match self {
            Self::Move(_) => TransformHandleKind::Move,
            Self::Rotate(_) => TransformHandleKind::Rotate,
            Self::Scale(_) => TransformHandleKind::Scale,
        }
    }
}
