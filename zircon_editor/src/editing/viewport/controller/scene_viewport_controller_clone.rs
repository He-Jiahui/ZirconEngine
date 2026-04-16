use super::SceneViewportController;

impl Clone for SceneViewportController {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            handles: self.handles.clone(),
            pointer_bridge: self.pointer_bridge.clone(),
        }
    }
}
