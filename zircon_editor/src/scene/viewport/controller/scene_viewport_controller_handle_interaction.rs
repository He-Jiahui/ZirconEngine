use zircon_framework::render::{HandleOverlayExtract, ViewportCameraSnapshot};
use zircon_math::Vec2;
use zircon_scene::Scene;

use crate::GizmoAxis;

use super::{viewport_drag_session::ViewportDragSession, SceneViewportController};

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn handle_overlays(
        &self,
        scene: &Scene,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        self.handles
            .build_overlays(scene, self.selected_node(), &self.state.settings, camera)
    }

    pub(in crate::scene::viewport::controller) fn begin_handle_drag(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let camera = self.current_camera(scene);
        let Some(session) = self.handles.begin_drag(
            scene,
            self.selected_node(),
            &self.state.settings,
            &camera,
            cursor,
            axis,
        ) else {
            return false;
        };

        self.state.drag = Some(ViewportDragSession::Handle { session });
        self.state.hover.hovered_axis = Some(axis);
        true
    }

    pub(in crate::scene::viewport::controller) fn end_handle_drag(&mut self) {
        let Some(ViewportDragSession::Handle { session }) = self.state.drag.take() else {
            return;
        };
        self.handles.end_drag(session);
    }
}
