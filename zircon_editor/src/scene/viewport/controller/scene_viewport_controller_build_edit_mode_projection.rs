use crate::scene::viewport::edit_mode_projection::build_scene_edit_mode_projection;
use crate::scene::viewport::SceneEditModeProjection;
use zircon_runtime::scene::Scene;

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn build_edit_mode_projection(&self, scene: &Scene) -> SceneEditModeProjection {
        build_scene_edit_mode_projection(
            scene,
            &self.state.settings,
            self.selected_node(),
            self.is_handle_drag_active(),
        )
    }
}
