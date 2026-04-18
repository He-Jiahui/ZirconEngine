use zircon_framework::render::FrameHistoryHandle;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn release_history(&mut self, handle: FrameHistoryHandle) {
        self.history_targets.remove(&handle);
    }
}
