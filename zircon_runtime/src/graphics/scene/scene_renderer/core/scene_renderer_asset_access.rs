use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn asset_manager_for_runtime_extract(
        &self,
    ) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
        self.streamer.asset_manager()
    }
}
