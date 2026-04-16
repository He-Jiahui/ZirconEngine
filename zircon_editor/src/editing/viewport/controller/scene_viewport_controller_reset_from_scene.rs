use zircon_math::Vec3;
use zircon_scene::Scene;

use crate::editing::viewport::pointer::ViewportOverlayPointerBridge;

use super::{viewport_hover_state::ViewportHoverState, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn reset_from_scene(&mut self, scene: Option<&Scene>) {
        self.reset_camera_from_scene(scene);
        self.state.orbit_target = scene
            .and_then(Self::selected_world_position)
            .unwrap_or(Vec3::ZERO);
        self.state.hover = ViewportHoverState::default();
        self.state.drag = None;
        self.pointer_bridge = ViewportOverlayPointerBridge::new();
    }
}
