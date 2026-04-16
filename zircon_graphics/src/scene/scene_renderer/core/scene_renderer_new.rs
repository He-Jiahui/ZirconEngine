use std::collections::HashMap;
use std::sync::Arc;

use zircon_asset::ProjectAssetManager;

use crate::types::GraphicsError;

use super::super::super::resources::ResourceStreamer;
use super::super::overlay::{EmptyViewportIconSource, ViewportIconSource};
use super::constants::OFFSCREEN_FORMAT;
use super::scene_renderer::SceneRenderer;
use super::scene_renderer_core::SceneRendererCore;

impl SceneRenderer {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source(asset_manager, Arc::new(EmptyViewportIconSource))
    }

    pub fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<Self, GraphicsError> {
        let backend = crate::backend::RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new_with_icon_source(
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            icon_source,
        );
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            core,
            streamer,
            target: None,
            history_targets: HashMap::new(),
            generation: 0,
            last_hybrid_gi_gpu_readback: None,
            last_virtual_geometry_gpu_readback: None,
        })
    }
}
