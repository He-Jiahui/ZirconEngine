use zircon_math::UVec2;

use crate::editing::viewport::handles::HandleToolRegistry;
use crate::editing::viewport::pointer::ViewportOverlayPointerBridge;

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
