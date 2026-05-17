use crate::scene::viewport::{HandleOverlayExtract, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec2;

use crate::scene::viewport::GizmoAxis;

use super::{viewport_drag_session::ViewportDragSession, SceneViewportController};

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn projected_selected_node(
        &self,
        scene: &Scene,
    ) -> Option<u64> {
        scene
            .editor_projection(self.selected_node())
            .selected_entity
    }

    pub(in crate::scene::viewport::controller) fn handle_overlays(
        &self,
        scene: &Scene,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let selected = self.projected_selected_node(scene);
        self.handles
            .build_overlays(scene, selected, &self.state.settings, camera)
    }

    pub(in crate::scene::viewport::controller) fn begin_handle_drag(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let Some(session) =
            self.handles
                .begin_drag(scene, selected, &self.state.settings, &camera, cursor, axis)
        else {
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
