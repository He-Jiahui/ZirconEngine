use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerBridge;

use super::{scene_viewport_state::SceneViewportState, SceneViewportController};

impl SceneViewportController {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            state: SceneViewportState::new(viewport_size),
            handles: HandleToolRegistry::default(),
            pointer_bridge: ViewportOverlayPointerBridge::new(),
        }
    }
}
