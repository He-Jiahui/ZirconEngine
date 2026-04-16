use crate::editing::viewport::handles::HandleToolRegistry;
use crate::editing::viewport::pointer::ViewportOverlayPointerBridge;

use super::scene_viewport_state::SceneViewportState;

pub(crate) struct SceneViewportController {
    pub(in crate::editing::viewport::controller) state: SceneViewportState,
    pub(in crate::editing::viewport::controller) handles: HandleToolRegistry,
    pub(in crate::editing::viewport::controller) pointer_bridge: ViewportOverlayPointerBridge,
}
