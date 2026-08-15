use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES,
    DEFAULT_GPU_TIMER_MAX_PASSES,
};
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration, RuntimePrepareCollectorRegistration,
};
use crate::plugin::PluginShaderModuleSource;

use crate::graphics::types::GraphicsError;

use super::super::super::super::resources::ResourceStreamer;
use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::super::super::overlay::{EmptyViewportIconSource, ViewportIconSource};
use super::super::constants::FINAL_COLOR_FORMAT;
use super::super::scene_renderer::SceneRendererAdvancedPluginOutputs;
use super::super::scene_renderer::{
    SceneRenderer, SceneRendererFrameTimingReport, SceneRendererStartupOptions,
    SceneRendererStartupReport,
};
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRenderer {
    pub fn new_with_startup_report(
        asset_manager: ProjectAssetManagerAccess,
    ) -> Result<(Self, SceneRendererStartupReport), GraphicsError> {
        Self::new_with_startup_options_and_report(
            asset_manager,
            SceneRendererStartupOptions::default(),
        )
    }

    pub fn new_with_startup_options_and_report(
        asset_manager: ProjectAssetManagerAccess,
        startup_options: SceneRendererStartupOptions,
    ) -> Result<(Self, SceneRendererStartupReport), GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features_and_shading_models_with_startup_report(
            asset_manager,
            Arc::new(EmptyViewportIconSource),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            startup_options,
        )
    }

    pub(crate) fn new_with_icon_source(
        asset_manager: ProjectAssetManagerAccess,
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
        asset_manager: ProjectAssetManagerAccess,
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
            Vec::new(),
        )
    }

    pub(crate) fn new_with_icon_source_and_plugin_render_features_and_shading_models(
        asset_manager: ProjectAssetManagerAccess,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        plugin_shader_module_sources: impl IntoIterator<Item = PluginShaderModuleSource>,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_icon_source_and_plugin_render_features_and_shading_models_with_startup_report(
            asset_manager,
            icon_source,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            plugin_geometry_sources,
            plugin_shading_models,
            plugin_shader_module_sources,
            SceneRendererStartupOptions::default(),
        )
        .map(|(renderer, _)| renderer)
    }

    fn new_with_icon_source_and_plugin_render_features_and_shading_models_with_startup_report(
        asset_manager: ProjectAssetManagerAccess,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        plugin_shader_module_sources: impl IntoIterator<Item = PluginShaderModuleSource>,
        startup_options: SceneRendererStartupOptions,
    ) -> Result<(Self, SceneRendererStartupReport), GraphicsError> {
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let render_pass_executors = render_pass_executors.into_iter().collect::<Vec<_>>();
        let runtime_prepare_collectors = runtime_prepare_collectors.into_iter().collect::<Vec<_>>();
        let plugin_geometry_sources = plugin_geometry_sources.into_iter().collect::<Vec<_>>();
        let plugin_shading_models = plugin_shading_models.into_iter().collect::<Vec<_>>();
        let plugin_shader_module_sources =
            plugin_shader_module_sources.into_iter().collect::<Vec<_>>();
        let backend_started = Instant::now();
        let backend = crate::graphics::backend::RenderBackend::new_offscreen()?;
        let backend_initialization = backend_started.elapsed();
        let adapter_info = backend.adapter.get_info();
        let core_started = Instant::now();
        let (mut core, core_startup) = SceneRendererCore::new_with_icon_source(
            asset_manager.clone(),
            &backend.device,
            &backend.queue,
            FINAL_COLOR_FORMAT,
            backend.backend_name(),
            &adapter_info,
            icon_source,
            &render_features,
            plugin_geometry_sources,
            plugin_shading_models.iter().cloned(),
            runtime_prepare_collectors,
            startup_options.deferred_lighting_profile(),
        )?;
        let core_initialization = core_started.elapsed();
        let resource_streamer_started = Instant::now();
        let mut streamer = ResourceStreamer::new_with_plugin_shading_models_and_shader_modules(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
            plugin_shading_models,
            plugin_shader_module_sources,
        )?;
        let resource_streamer_initialization = resource_streamer_started.elapsed();
        core.mesh_pipelines
            .set_async_pipeline_compile_enabled(startup_options.async_pipeline_compile_enabled());
        let environment_only_pbr_base_prewarm =
            if startup_options.requires_environment_only_pbr_base_prewarm() {
                Some(
                    core.mesh_pipelines
                        .prewarm_environment_only_pbr_base_pipeline(&backend.device, &mut streamer)?
                        .into(),
                )
            } else if startup_options.queues_environment_only_pbr_base_prewarm() {
                Some(
                    core.mesh_pipelines
                        .queue_environment_only_pbr_base_pipeline(&backend.device, &mut streamer)?
                        .into(),
                )
            } else {
                None
            };
        let gpu_pass_timing_requested = startup_options.allow_gpu_timing();
        let gpu_pass_timer = gpu_pass_timing_requested
            .then(|| {
                GpuPassTimer::try_new(
                    &backend.device,
                    &backend.queue,
                    DEFAULT_GPU_TIMER_MAX_PASSES,
                )
            })
            .flatten();
        let gpu_pipeline_statistics_timer = gpu_pass_timing_requested
            .then(|| {
                GpuPipelineStatisticsTimer::try_new(
                    &backend.device,
                    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES,
                )
            })
            .flatten();
        let last_gpu_timing_status = if !gpu_pass_timing_requested {
            crate::core::framework::render::RenderGpuTimingStatus::Disabled
        } else if gpu_pass_timer.is_some() {
            crate::core::framework::render::RenderGpuTimingStatus::Deferred
        } else {
            crate::core::framework::render::RenderGpuTimingStatus::Unavailable
        };
        Ok((
            Self {
                backend,
                core,
                streamer,
                target: None,
                last_capture_target: None,
                history_targets: HashMap::new(),
                generation: 0,
                gpu_pass_timing_requested,
                gpu_pass_timer,
                gpu_pipeline_statistics_timer,
                last_gpu_timer_frame_result: None,
                last_gpu_timing_status,
                last_gpu_pipeline_statistics_frame_result: None,
                render_pass_executors:
                    RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
                        render_features,
                        render_pass_executors,
                )
                .with_environment_ibl_bake_compute_executors(),
                last_render_graph_execution: RenderGraphExecutionRecord::default(),
                last_prepared_mesh_queue_stats: Default::default(),
                last_prepared_sprite_queue_stats: Default::default(),
                frame_timing_report_requested: false,
                parallel_record_min_passes_per_bucket: None,
                hzb_diagnostics_readback_enabled: false,
                last_frame_timing_report: SceneRendererFrameTimingReport::default(),
                advanced_plugin_outputs: SceneRendererAdvancedPluginOutputs::default(),
            },
            SceneRendererStartupReport {
                backend_initialization,
                core_initialization,
                core_startup,
                resource_streamer_initialization,
                environment_only_pbr_base_prewarm,
            },
        ))
    }
}
