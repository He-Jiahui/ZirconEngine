use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::RenderFeatureDescriptor;

use crate::graphics::types::GraphicsError;

use super::super::super::super::resources::ResourceStreamer;
use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::super::super::overlay::ViewportIconSource;
use super::super::constants::OFFSCREEN_FORMAT;
use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer::SceneRendererAdvancedPluginOutputs;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRenderer {
    pub(crate) fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features(
            asset_manager,
            icon_source,
            Vec::new(),
        )
    }

    pub(crate) fn new_with_icon_source_and_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Result<Self, GraphicsError> {
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let backend = crate::graphics::backend::RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new_with_icon_source(
            asset_manager.clone(),
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            icon_source,
            &render_features,
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
            render_pass_executors:
                RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features(
                    render_features,
                ),
            last_render_graph_execution: RenderGraphExecutionRecord::default(),
            advanced_plugin_outputs: SceneRendererAdvancedPluginOutputs::default(),
        })
    }
}
