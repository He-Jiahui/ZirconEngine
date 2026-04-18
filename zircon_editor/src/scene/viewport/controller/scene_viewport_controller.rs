use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerBridge;

use super::scene_viewport_state::SceneViewportState;

pub(crate) struct SceneViewportController {
    pub(in crate::scene::viewport::controller) state: SceneViewportState,
    pub(in crate::scene::viewport::controller) handles: HandleToolRegistry,
    pub(in crate::scene::viewport::controller) pointer_bridge: ViewportOverlayPointerBridge,
}
