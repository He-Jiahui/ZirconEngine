use super::handle_drag_session::HandleDragSession;

impl HandleDragSession {
    pub(crate) fn node_id(&self) -> u64 {
        match self {
            Self::Move(session) | Self::Rotate(session) | Self::Scale(session) => session.node_id,
        }
    }
}
