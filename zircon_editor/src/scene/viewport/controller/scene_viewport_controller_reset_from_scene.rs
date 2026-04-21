use zircon_runtime::core::math::Vec3;
use zircon_runtime::scene::Scene;

use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;

use super::{viewport_hover_state::ViewportHoverState, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn reset_from_scene(&mut self, scene: Option<&Scene>) {
        self.reset_camera_from_scene(scene);
        self.state.orbit_target = scene
            .and_then(|scene| Self::selected_world_position(scene, self.selected_node()))
            .unwrap_or(Vec3::ZERO);
        self.state.hover = ViewportHoverState::default();
        self.state.drag = None;
        self.pointer_bridge = ViewportOverlayPointerRouter::new();
    }
}
