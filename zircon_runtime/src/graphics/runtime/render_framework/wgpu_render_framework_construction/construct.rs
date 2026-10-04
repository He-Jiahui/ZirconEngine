use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[cfg(test)]
use crate::asset::ProjectAssetManager;
use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{
    AdvancedProviderAvailability, GeometrySourceDescriptor, ShadingModelDescriptor,
};
use crate::core::TaskPool;
use crate::graphics::pipeline::CompiledGraphCache;
use crate::graphics::{
    GraphicsError, SceneRenderer, SceneRendererStartupOptions, SceneRendererStartupReport,
};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::PluginShaderModuleSource;
use crate::text::font::{shared_font_collection_service, FontCollectionService};

use super::super::capability_summary::{capability_summary, render_device_diagnostics};
use super::super::graphics_debugger_capture::{
    renderdoc_capture_frame_count_from_env, GraphicsDebuggerState,
};
use super::super::render_framework_state::RenderFrameworkState;
use super::super::wgpu_render_framework::{WgpuRenderFramework, WgpuRenderFrameworkCore};
use super::create_default_pipelines::create_default_pipelines;

impl WgpuRenderFramework {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        let asset_manager = ProjectAssetManagerAccess::for_test(asset_manager);
        let compute_task_pool = asset_manager.test_worker_pool();
        Self::new(asset_manager, compute_task_pool)
    }

    /// Standalone compatibility constructor for tests and diagnostic products.
    ///
    /// Core-owned Runtime products construct the framework through the builtin Graphics module
    /// host so the renderer inherits that Core's `TextRuntimeContext`.
    pub fn new(
        asset_manager: ProjectAssetManagerAccess,
        compute_task_pool: TaskPool,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_features(
            asset_manager,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            compute_task_pool,
        )
    }

    /// Creates the framework around a renderer with a specialized startup profile.
    ///
    /// Diagnostic products use this entry point when they need the renderer's
    /// startup report without taking ownership of the renderer or its surfaces.
    pub fn new_with_startup_options_and_report(
        asset_manager: ProjectAssetManagerAccess,
        startup_options: SceneRendererStartupOptions,
        compute_task_pool: TaskPool,
    ) -> Result<(Self, SceneRendererStartupReport), GraphicsError> {
        let (renderer, startup_report) =
            SceneRenderer::new_with_startup_options_and_report(asset_manager, startup_options)?;
        let framework =
            Self::from_renderer(renderer, Vec::new(), None, None, None, compute_task_pool);
        Ok((framework, startup_report))
    }

    pub fn new_with_plugin_render_features(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        compute_task_pool: TaskPool,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions(
            asset_manager,
            render_features,
            render_pass_executors,
            Vec::new(),
            Vec::new(),
            virtual_geometry_runtime_providers,
            compute_task_pool,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Result<Self, GraphicsError> {
        let asset_manager = ProjectAssetManagerAccess::for_test(asset_manager);
        let compute_task_pool = asset_manager.test_worker_pool();
        Self::new_with_plugin_render_features(
            asset_manager,
            render_features,
            render_pass_executors,
            virtual_geometry_runtime_providers,
            compute_task_pool,
        )
    }

    pub fn new_with_plugin_render_extensions(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        compute_task_pool: TaskPool,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_shading_models(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            Vec::new(),
            Vec::new(),
            hybrid_gi_runtime_providers,
            virtual_geometry_runtime_providers,
            compute_task_pool,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_extensions(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Result<Self, GraphicsError> {
        let asset_manager = ProjectAssetManagerAccess::for_test(asset_manager);
        let compute_task_pool = asset_manager.test_worker_pool();
        Self::new_with_plugin_render_extensions(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            virtual_geometry_runtime_providers,
            compute_task_pool,
        )
    }

    pub fn new_with_plugin_render_extensions_and_shading_models(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        compute_task_pool: TaskPool,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_solari_and_shading_models(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            Vec::new(),
            virtual_geometry_runtime_providers,
            plugin_geometry_sources,
            plugin_shading_models,
            Vec::new(),
            compute_task_pool,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_extensions_and_shading_models(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Result<Self, GraphicsError> {
        let asset_manager = ProjectAssetManagerAccess::for_test(asset_manager);
        let compute_task_pool = asset_manager.test_worker_pool();
        Self::new_with_plugin_render_extensions_and_shading_models(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            plugin_geometry_sources,
            plugin_shading_models,
            hybrid_gi_runtime_providers,
            virtual_geometry_runtime_providers,
            compute_task_pool,
        )
    }

    pub fn new_with_plugin_render_extensions_and_solari_and_shading_models(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        plugin_shader_module_sources: impl IntoIterator<Item = PluginShaderModuleSource>,
        compute_task_pool: TaskPool,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_solari_and_compute_task_pool(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            solari_runtime_providers,
            virtual_geometry_runtime_providers,
            plugin_geometry_sources,
            plugin_shading_models,
            plugin_shader_module_sources,
            compute_task_pool,
            // Runtime201 standalone compatibility: Core-owned products inject TextRuntimeContext.
            shared_font_collection_service(),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_plugin_render_extensions_and_solari_and_shading_models(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Result<Self, GraphicsError> {
        let asset_manager = ProjectAssetManagerAccess::for_test(asset_manager);
        let compute_task_pool = asset_manager.test_worker_pool();
        Self::new_with_plugin_render_extensions_and_solari_and_shading_models(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            solari_runtime_providers,
            virtual_geometry_runtime_providers,
            plugin_geometry_sources,
            plugin_shading_models,
            Vec::new(),
            compute_task_pool,
        )
    }

    pub(crate) fn new_with_plugin_render_extensions_and_solari_and_compute_task_pool(
        asset_manager: ProjectAssetManagerAccess,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        plugin_shader_module_sources: impl IntoIterator<Item = PluginShaderModuleSource>,
        compute_task_pool: TaskPool,
        font_collection: Arc<FontCollectionService>,
    ) -> Result<Self, GraphicsError> {
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let render_pass_executors = render_pass_executors.into_iter().collect::<Vec<_>>();
        let runtime_prepare_collectors = runtime_prepare_collectors.into_iter().collect::<Vec<_>>();
        let hybrid_gi_runtime_providers =
            hybrid_gi_runtime_providers.into_iter().collect::<Vec<_>>();
        let solari_runtime_providers = solari_runtime_providers.into_iter().collect::<Vec<_>>();
        let virtual_geometry_runtime_providers = virtual_geometry_runtime_providers
            .into_iter()
            .collect::<Vec<_>>();
        let plugin_geometry_sources = plugin_geometry_sources.into_iter().collect::<Vec<_>>();
        let plugin_shading_models = plugin_shading_models.into_iter().collect::<Vec<_>>();
        let plugin_shader_module_sources =
            plugin_shader_module_sources.into_iter().collect::<Vec<_>>();
        let selected_hybrid_gi_runtime_provider =
            select_hybrid_gi_runtime_provider(hybrid_gi_runtime_providers)?;
        let selected_solari_runtime_provider =
            select_solari_runtime_provider(solari_runtime_providers)?;
        let selected_virtual_geometry_runtime_provider =
            select_virtual_geometry_runtime_provider(virtual_geometry_runtime_providers)?;
        let renderer = SceneRenderer::new_with_plugin_render_extensions_and_shading_models_and_font_collection(
            asset_manager,
            render_features.clone(),
            render_pass_executors,
            runtime_prepare_collectors,
            plugin_geometry_sources,
            plugin_shading_models,
            plugin_shader_module_sources,
            font_collection,
        )?;
        Ok(Self::from_renderer(
            renderer,
            render_features,
            selected_hybrid_gi_runtime_provider,
            selected_solari_runtime_provider,
            selected_virtual_geometry_runtime_provider,
            compute_task_pool,
        ))
    }

    fn from_renderer(
        renderer: SceneRenderer,
        render_features: Vec<RenderFeatureDescriptor>,
        selected_hybrid_gi_runtime_provider: Option<HybridGiRuntimeProviderRegistration>,
        selected_solari_runtime_provider: Option<SolariRuntimeProviderRegistration>,
        selected_virtual_geometry_runtime_provider: Option<
            VirtualGeometryRuntimeProviderRegistration,
        >,
        compute_task_pool: TaskPool,
    ) -> Self {
        let advanced_provider_availability = selected_advanced_provider_availability(
            selected_hybrid_gi_runtime_provider.as_ref(),
            selected_virtual_geometry_runtime_provider.as_ref(),
        );
        let device_fault_gate = renderer.device_fault_gate();
        let backend_caps = renderer.backend_caps();
        let render_capabilities = capability_summary(&backend_caps);
        let device_diagnostics = render_device_diagnostics(&backend_caps);
        let graphics_debugger = GraphicsDebuggerState::available_with_capture_frame_count(
            renderer.backend_name(),
            renderdoc_capture_frame_count_from_env(),
        );
        Self {
            submission_scheduler: Mutex::new(Default::default()),
            core: Arc::new(WgpuRenderFrameworkCore {
                device_fault_gate,
                operation_lock: Mutex::new(()),
                compute_task_pool,
                planar_reflection_updates: Mutex::new(Default::default()),
                environment_captures: Mutex::new(Default::default()),
                state: Mutex::new(RenderFrameworkState {
                    renderer,
                    pending_environment_capture_submission: None,
                    environment_capture_residency: Default::default(),
                    last_retained_scene_color_viewport: None,
                    next_viewport_id: 1,
                    next_history_id: 1,
                    pipelines: create_default_pipelines(&render_features),
                    compiled_graph_cache: CompiledGraphCache::default(),
                    environment_ibl_hydration_cache: Default::default(),
                    hybrid_gi_runtime_provider: selected_hybrid_gi_runtime_provider,
                    solari_runtime_provider: selected_solari_runtime_provider,
                    virtual_geometry_runtime_provider: selected_virtual_geometry_runtime_provider,
                    last_virtual_geometry_debug_snapshot: None,
                    viewports: HashMap::new(),
                    stats: crate::core::framework::render::RenderStats {
                        device_diagnostics,
                        capabilities: render_capabilities,
                        advanced_provider_availability,
                        ..crate::core::framework::render::RenderStats::default()
                    },
                    frame_profiler: Default::default(),
                    memory_budget: Default::default(),
                    degrade_ladder: Default::default(),
                    graphics_debugger,
                    viewport_products: Default::default(),
                    viewport_pick_frames: Default::default(),
                    viewport_picks: Default::default(),
                }),
            }),
        }
    }
}

fn select_hybrid_gi_runtime_provider(
    providers: Vec<HybridGiRuntimeProviderRegistration>,
) -> Result<Option<HybridGiRuntimeProviderRegistration>, GraphicsError> {
    select_provider(
        "hybrid_global_illumination",
        providers,
        HybridGiRuntimeProviderRegistration::provider_id,
        HybridGiRuntimeProviderRegistration::priority,
    )
}

fn select_solari_runtime_provider(
    providers: Vec<SolariRuntimeProviderRegistration>,
) -> Result<Option<SolariRuntimeProviderRegistration>, GraphicsError> {
    select_provider(
        "solari",
        providers,
        SolariRuntimeProviderRegistration::provider_id,
        SolariRuntimeProviderRegistration::priority,
    )
}

fn select_virtual_geometry_runtime_provider(
    providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
) -> Result<Option<VirtualGeometryRuntimeProviderRegistration>, GraphicsError> {
    select_provider(
        "virtual_geometry",
        providers,
        VirtualGeometryRuntimeProviderRegistration::provider_id,
        VirtualGeometryRuntimeProviderRegistration::priority,
    )
}

fn selected_advanced_provider_availability(
    hybrid_gi: Option<&HybridGiRuntimeProviderRegistration>,
    virtual_geometry: Option<&VirtualGeometryRuntimeProviderRegistration>,
) -> AdvancedProviderAvailability {
    let availability = AdvancedProviderAvailability::new();
    let availability = match virtual_geometry {
        Some(provider) => availability.with_virtual_geometry_provider(provider.provider_id()),
        None => availability,
    };
    match hybrid_gi {
        Some(provider) => availability.with_hybrid_gi_provider(provider.provider_id()),
        None => availability,
    }
}

fn select_provider<T>(
    feature_label: &str,
    providers: Vec<T>,
    provider_id: impl Fn(&T) -> &str,
    priority: impl Fn(&T) -> i32,
) -> Result<Option<T>, GraphicsError> {
    if providers.is_empty() {
        return Ok(None);
    }

    let mut seen_provider_ids = HashSet::new();
    for provider in &providers {
        let id = provider_id(provider);
        if !seen_provider_ids.insert(id) {
            return Err(GraphicsError::AdvancedProviderSelection(format!(
                "{feature_label} provider `{id}` registered more than once"
            )));
        }
    }
    drop(seen_provider_ids);

    let mut best_index = 0;
    let mut best_priority = priority(&providers[0]);
    let mut tied_best_index = None;
    for (index, provider) in providers.iter().enumerate().skip(1) {
        let candidate_priority = priority(provider);
        if candidate_priority > best_priority {
            best_index = index;
            best_priority = candidate_priority;
            tied_best_index = None;
        } else if candidate_priority == best_priority {
            tied_best_index = Some(index);
        }
    }

    if let Some(tied_index) = tied_best_index {
        return Err(GraphicsError::AdvancedProviderSelection(format!(
            "{feature_label} provider priority tie at {best_priority}: `{}` and `{}`",
            provider_id(&providers[best_index]),
            provider_id(&providers[tied_index])
        )));
    }

    Ok(providers.into_iter().nth(best_index))
}

#[cfg(test)]
#[path = "tests/construct.rs"]
mod tests;
