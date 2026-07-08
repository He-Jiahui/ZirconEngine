use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration, RuntimePrepareCollectorRegistration,
};

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
            Vec::new(),
            Vec::new(),
        )
    }

    pub(crate) fn new_with_icon_source_and_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features_and_shading_models(
            asset_manager,
            icon_source,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            Vec::new(),
            Vec::new(),
        )
    }

    pub(crate) fn new_with_icon_source_and_plugin_render_features_and_shading_models(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let render_pass_executors = render_pass_executors.into_iter().collect::<Vec<_>>();
        let runtime_prepare_collectors = runtime_prepare_collectors.into_iter().collect::<Vec<_>>();
        let plugin_geometry_sources = plugin_geometry_sources.into_iter().collect::<Vec<_>>();
        let plugin_shading_models = plugin_shading_models.into_iter().collect::<Vec<_>>();
        let backend = crate::graphics::backend::RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new_with_icon_source(
            asset_manager.clone(),
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            backend.backend_name(),
            icon_source,
            &render_features,
            plugin_geometry_sources,
            plugin_shading_models.iter().cloned(),
            runtime_prepare_collectors,
        )?;
        let streamer = ResourceStreamer::new_with_plugin_shading_models(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
            plugin_shading_models,
        )?;

        Ok(Self {
            backend,
            core,
            streamer,
            target: None,
            history_targets: HashMap::new(),
            generation: 0,
            render_pass_executors:
                RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
                    render_features,
                    render_pass_executors,
            )
            .with_environment_ibl_bake_compute_executors(),
            last_render_graph_execution: RenderGraphExecutionRecord::default(),
            last_prepared_mesh_queue_stats: Default::default(),
            last_prepared_sprite_queue_stats: Default::default(),
            advanced_plugin_outputs: SceneRendererAdvancedPluginOutputs::default(),
        })
    }
}
