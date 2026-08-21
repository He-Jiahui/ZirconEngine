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
mod tests {
    use crate::scene::viewport::{SceneViewportController, ViewportInput};
    use zircon_runtime::scene::Scene;
    use zircon_runtime_interface::math::{UVec2, Vec2};

    #[test]
    fn cancellation_terminates_active_camera_navigation() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let mut scene = Scene::new();

        controller.handle_input(&mut scene, ViewportInput::RightPressed(Vec2::ZERO));

        assert!(controller.cancel_interaction());
        let feedback = controller.handle_input(
            &mut scene,
            ViewportInput::PointerMoved(Vec2::new(120.0, 48.0)),
        );
        assert!(!feedback.camera_updated);
    }
}
