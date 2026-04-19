use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;

use crate::graphics::types::GraphicsError;

use super::super::super::overlay::EmptyViewportIconSource;
use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source(asset_manager, Arc::new(EmptyViewportIconSource))
    }
}
