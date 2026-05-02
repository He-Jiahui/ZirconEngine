use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;

use super::{scene_viewport_state::SceneViewportState, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            state: SceneViewportState::new(viewport_size),
            handles: HandleToolRegistry::default(),
            pointer_bridge: ViewportOverlayPointerRouter::new(),
        }
    }
}
