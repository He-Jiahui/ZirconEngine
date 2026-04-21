use crate::scene::viewport::{GizmoAxis, SceneViewportSettings, ViewportState};
use zircon_runtime::core::math::Vec3;

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

    pub(crate) fn selected_node(&self) -> Option<u64> {
        self.state.selected
    }

    pub(crate) fn set_selected_node(&mut self, node_id: Option<u64>) -> bool {
        if self.state.selected == node_id {
            return false;
        }
        self.state.selected = node_id;
        true
    }

    pub(crate) fn set_orbit_target(&mut self, target: Vec3) {
        self.state.orbit_target = target;
    }

    pub(crate) fn is_handle_drag_active(&self) -> bool {
        matches!(
            self.state.drag,
            Some(super::viewport_drag_session::ViewportDragSession::Handle { .. })
        )
    }
}
