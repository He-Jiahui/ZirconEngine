use crate::scene::viewport::{HandleOverlayExtract, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Transform, Vec2};

use crate::core::editing::interactive_transform::selection_pivot_transform;
use crate::scene::viewport::handles::HandleSelection;
use crate::scene::viewport::GizmoAxis;

use super::{viewport_drag_session::ViewportDragSession, SceneViewportController};

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn projected_selected_node(
        &self,
        scene: &Scene,
    ) -> Option<u64> {
        self.state
            .selection
            .active_primary()
            .filter(|entity| scene.contains_entity(*entity))
    }

    pub(in crate::scene::viewport::controller) fn handle_overlays(
        &self,
        scene: &Scene,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let selected = self.selected_handle_transform(scene);
        self.handle_overlays_for_transform(selected, camera)
    }

    pub(crate) fn handle_overlays_for_transform(
        &self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let handle_kind = self.base_transform_handle();
        self.handles.build_overlays(
            selected.map(|(entity, transform)| HandleSelection { entity, transform }),
            &self.state.settings,
            handle_kind,
            camera,
        )
    }

    pub(crate) fn handle_axis_at_cursor_for_transform(
        &self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
    ) -> Option<GizmoAxis> {
        if !self.state.settings.gizmos_enabled {
            return None;
        }
        let overlays = self.handle_overlays_for_transform(selected, camera);
        match crate::scene::viewport::pointer::local_handle_route(
            &overlays,
            camera,
            self.state.viewport.size,
            cursor,
        ) {
            Some(crate::scene::viewport::pointer::ViewportPointerRoute::HandleAxis {
                axis,
                ..
            }) => Some(axis),
            _ => None,
        }
    }

    pub(in crate::scene::viewport::controller) fn begin_handle_drag(
        &mut self,
        scene: &Scene,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let camera = self.current_camera(scene);
        let selected = self.selected_handle_transform(scene);
        self.begin_handle_drag_for_transform(selected, &camera, cursor, axis)
    }

    pub(crate) fn begin_handle_drag_for_transform(
        &mut self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> bool {
        let snap_steps = self.snap_steps();
        let handle_kind = self.base_transform_handle();
        let Some(session) = self.handles.begin_drag(
            selected.map(|(entity, transform)| HandleSelection { entity, transform }),
            &self.state.settings,
            handle_kind,
            snap_steps,
            camera,
            cursor,
            axis,
        ) else {
            return false;
        };

        self.state.drag = Some(ViewportDragSession::Handle { session });
        self.state.hover.hovered_axis = Some(axis);
        true
    }

    pub(crate) fn update_handle_drag_for_transform(
        &mut self,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
    ) -> Option<crate::scene::viewport::ViewportTransformRequest> {
        let Some(ViewportDragSession::Handle { mut session }) = self.state.drag.take() else {
            return None;
        };
        let preview = self
            .handles
            .update_drag(&mut session, camera, self.state.viewport.size, cursor)
            .map(
                |target_pivot_world| crate::scene::viewport::ViewportTransformRequest {
                    primary: session.node_id(),
                    target_pivot_world,
                },
            );
        self.state.drag = Some(ViewportDragSession::Handle { session });
        preview
    }

    pub(crate) fn finish_handle_drag_for_transform(&mut self) -> bool {
        let Some(ViewportDragSession::Handle { session }) = self.state.drag.take() else {
            return false;
        };
        self.handles.end_drag(session);
        true
    }

    pub(crate) fn set_handle_hover_for_transform(&mut self, axis: Option<GizmoAxis>) -> bool {
        let changed = self.state.hover.hovered_axis != axis;
        self.state.hover.hovered_axis = axis;
        changed
    }

    pub(crate) fn active_interactive_transform_spec(
        &self,
    ) -> Option<crate::core::editing::interactive_transform::InteractiveTransformSpec> {
        match self.state.drag.as_ref()? {
            ViewportDragSession::Handle { session } => Some(session.interactive_transform_spec()),
            _ => None,
        }
    }

    fn selected_handle_transform(&self, scene: &Scene) -> Option<(u64, Transform)> {
        let primary = self.projected_selected_node(scene)?;
        selection_pivot_transform(
            scene,
            self.state.selection.active_items().iter().copied(),
            primary,
            self.interactive_transform_pivot_mode(),
        )
    }

    pub(in crate::scene::viewport::controller) fn end_handle_drag(&mut self) {
        let Some(ViewportDragSession::Handle { session }) = self.state.drag.take() else {
            return;
        };
        self.handles.end_drag(session);
    }
}

#[cfg(test)]
#[path = "tests/scene_viewport_controller_handle_interaction.rs"]
mod tests;
