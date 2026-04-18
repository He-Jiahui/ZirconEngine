use zircon_scene::SceneViewportTool;

use super::handle_drag_session::HandleDragSession;

impl HandleDragSession {
    pub(in crate::scene::viewport::handles) fn tool(&self) -> SceneViewportTool {
        match self {
            Self::Move(_) => SceneViewportTool::Move,
            Self::Rotate(_) => SceneViewportTool::Rotate,
            Self::Scale(_) => SceneViewportTool::Scale,
        }
    }
}
