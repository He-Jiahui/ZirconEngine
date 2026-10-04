use super::SceneViewportController;

impl std::fmt::Debug for SceneViewportController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneViewportController")
            .field("state", &self.state)
            .field("handles", &self.handles)
            .field("overlay_providers", &self.overlay_providers)
            .finish()
    }
}
