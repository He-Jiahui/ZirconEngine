use crate::core::framework::render::FrameHistoryHandle;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    /// Runtime 在相机/viewport 生命周期结束时释放对应历史域；下一次渲染须重新建立有效历史。
    pub(crate) fn release_history(&mut self, handle: FrameHistoryHandle) {
        self.history_targets.remove(&handle);
    }
}
