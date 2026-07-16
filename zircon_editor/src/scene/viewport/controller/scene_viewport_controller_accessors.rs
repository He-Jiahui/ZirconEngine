use crate::scene::selection::SelectionModel;
use crate::scene::viewport::{GizmoAxis, SceneViewportSettings, ViewportState};
use zircon_runtime_interface::math::Vec3;

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn viewport(&self) -> &ViewportState {
        &self.state.viewport
    }

    pub(crate) fn clone_for_render(&self) -> Self {
        self.clone()
    }

    pub(crate) fn hovered_axis(&self) -> Option<GizmoAxis> {
        self.state.hover.hovered_axis
    }

    pub(crate) fn settings(&self) -> &SceneViewportSettings {
        &self.state.settings
    }

    pub(crate) fn settings_mut(&mut self) -> &mut SceneViewportSettings {
        &mut self.state.settings
    }

    pub(crate) fn selection(&self) -> &SelectionModel {
        &self.state.selection
    }

    pub(crate) fn selection_mut(&mut self) -> &mut SelectionModel {
        &mut self.state.selection
    }

    pub(crate) fn set_orbit_target(&mut self, target: Vec3) {
        self.state.orbit_target = target;
        self.state.orbit_controller.set_target(target);
    }

    pub(crate) fn is_handle_drag_active(&self) -> bool {
        matches!(
            self.state.drag,
            Some(super::viewport_drag_session::ViewportDragSession::Handle { .. })
        )
    }
}
