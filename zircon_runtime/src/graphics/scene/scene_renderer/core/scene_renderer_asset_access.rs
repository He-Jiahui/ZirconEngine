use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    /// Runtime extract 借用与 renderer streamer 同源的项目资源管理器，避免渲染与提取使用不同资源世代。
    pub(crate) fn asset_manager_for_runtime_extract(
        &self,
    ) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
        self.streamer.asset_manager()
    }
}
