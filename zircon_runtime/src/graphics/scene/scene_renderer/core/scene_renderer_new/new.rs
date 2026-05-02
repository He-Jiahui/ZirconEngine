use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::{RenderFeatureDescriptor, RenderPassExecutorRegistration};

use crate::graphics::types::GraphicsError;

use super::super::super::overlay::EmptyViewportIconSource;
use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source(asset_manager, Arc::new(EmptyViewportIconSource))
    }

    pub fn new_with_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features(
            asset_manager,
            Arc::new(EmptyViewportIconSource),
            render_features,
            render_pass_executors,
        )
    }
}
