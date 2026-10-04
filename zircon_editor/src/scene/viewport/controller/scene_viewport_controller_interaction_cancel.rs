use super::{viewport_drag_session::ViewportDragSession, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn cancel_interaction(&mut self) -> bool {
        let Some(drag) = self.state.drag.take() else {
            return false;
        };

        if let ViewportDragSession::Handle { session } = drag {
            self.handles.end_drag(session);
        }
        self.state.hover.hovered_axis = None;
        true
    }
}

#[cfg(test)]
#[path = "tests/scene_viewport_controller_interaction_cancel.rs"]
mod tests;
