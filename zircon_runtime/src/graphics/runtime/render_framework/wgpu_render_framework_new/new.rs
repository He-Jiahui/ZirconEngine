use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::AdvancedProviderAvailability;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::rhi::RenderDevice;
use crate::rhi_wgpu::WgpuRenderDevice;

use crate::{GraphicsError, SceneRenderer};

use super::super::capability_summary::capability_summary;
use super::super::graphics_debugger_capture::{
    renderdoc_capture_next_from_env, GraphicsDebuggerState,
};
use super::super::render_framework_state::RenderFrameworkState;
use super::super::wgpu_render_framework::WgpuRenderFramework;
use super::create_default_pipelines::create_default_pipelines;

impl WgpuRenderFramework {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_features(asset_manager, Vec::new(), Vec::new(), Vec::new())
    }

    pub fn new_with_plugin_render_features(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions(
            asset_manager,
            render_features,
            render_pass_executors,
            Vec::new(),
            Vec::new(),
            virtual_geometry_runtime_providers,
        )
    }

    pub fn new_with_plugin_render_extensions(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Result<Self, GraphicsError> {
        Self::new_with_plugin_render_extensions_and_solari(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            Vec::new(),
            virtual_geometry_runtime_providers,
        )
    }

    pub fn new_with_plugin_render_extensions_and_solari(
        asset_manager: Arc<ProjectAssetManager>,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
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
        let selected_hybrid_gi_runtime_provider =
            select_hybrid_gi_runtime_provider(hybrid_gi_runtime_providers)?;
        let selected_solari_runtime_provider =
            select_solari_runtime_provider(solari_runtime_providers)?;
        let selected_virtual_geometry_runtime_provider =
            select_virtual_geometry_runtime_provider(virtual_geometry_runtime_providers)?;
        let advanced_provider_availability = selected_advanced_provider_availability(
            selected_hybrid_gi_runtime_provider.as_ref(),
            selected_virtual_geometry_runtime_provider.as_ref(),
        );
        let render_device = WgpuRenderDevice::new_headless();
        let renderer = SceneRenderer::new_with_plugin_render_extensions(
            asset_manager,
            render_features.clone(),
            render_pass_executors,
            runtime_prepare_collectors,
        )?;
        let graphics_debugger = GraphicsDebuggerState::available_with_capture_next_created_viewport(
            renderer.backend_name(),
            renderdoc_capture_next_from_env(),
        );
        Ok(Self {
            operation_lock: Mutex::new(()),
            state: Mutex::new(RenderFrameworkState {
                renderer,
                next_viewport_id: 1,
                next_history_id: 1,
                pipelines: create_default_pipelines(&render_features),
                hybrid_gi_runtime_provider: selected_hybrid_gi_runtime_provider,
                solari_runtime_provider: selected_solari_runtime_provider,
                virtual_geometry_runtime_provider: selected_virtual_geometry_runtime_provider,
                last_virtual_geometry_debug_snapshot: None,
                viewports: HashMap::new(),
                stats: crate::core::framework::render::RenderStats {
                    capabilities: capability_summary(render_device.caps()),
                    advanced_provider_availability,
                    ..crate::core::framework::render::RenderStats::default()
                },
                graphics_debugger,
            }),
        })
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

fn select_provider<T: Clone>(
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
        if !seen_provider_ids.insert(id.to_string()) {
            return Err(GraphicsError::AdvancedProviderSelection(format!(
                "{feature_label} provider `{id}` registered more than once"
            )));
        }
    }

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

    Ok(Some(providers[best_index].clone()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::graphics::{
        HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
        HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration, HybridGiRuntimeState,
        HybridGiRuntimeUpdate, VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
        VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
        VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
        VirtualGeometryRuntimeUpdate,
    };

    use super::{
        select_hybrid_gi_runtime_provider, select_virtual_geometry_runtime_provider,
        selected_advanced_provider_availability,
    };

    #[derive(Debug)]
    struct NoopVirtualGeometryProvider;

    impl VirtualGeometryRuntimeProvider for NoopVirtualGeometryProvider {
        fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
            Box::new(NoopVirtualGeometryState)
        }
    }

    #[derive(Debug)]
    struct NoopVirtualGeometryState;

    impl VirtualGeometryRuntimeState for NoopVirtualGeometryState {
        fn prepare_frame(
            &mut self,
            _input: VirtualGeometryRuntimePrepareInput<'_>,
        ) -> VirtualGeometryRuntimePrepareOutput {
            VirtualGeometryRuntimePrepareOutput::default()
        }

        fn update_after_render(
            &mut self,
            _feedback: VirtualGeometryRuntimeFeedback,
        ) -> VirtualGeometryRuntimeUpdate {
            VirtualGeometryRuntimeUpdate::default()
        }
    }

    #[derive(Debug)]
    struct NoopHybridGiProvider;

    impl HybridGiRuntimeProvider for NoopHybridGiProvider {
        fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
            Box::new(NoopHybridGiState)
        }
    }

    struct NoopHybridGiState;

    impl HybridGiRuntimeState for NoopHybridGiState {
        fn prepare_frame(
            &mut self,
            _input: HybridGiRuntimePrepareInput<'_>,
        ) -> HybridGiRuntimePrepareOutput {
            HybridGiRuntimePrepareOutput::default()
        }

        fn update_after_render(
            &mut self,
            _feedback: HybridGiRuntimeFeedback,
        ) -> HybridGiRuntimeUpdate {
            HybridGiRuntimeUpdate::default()
        }
    }

    fn virtual_geometry_registration(
        provider_id: &str,
        priority: i32,
    ) -> VirtualGeometryRuntimeProviderRegistration {
        VirtualGeometryRuntimeProviderRegistration::new(
            provider_id,
            Arc::new(NoopVirtualGeometryProvider),
        )
        .with_priority(priority)
    }

    fn hybrid_gi_registration(
        provider_id: &str,
        priority: i32,
    ) -> HybridGiRuntimeProviderRegistration {
        HybridGiRuntimeProviderRegistration::new(provider_id, Arc::new(NoopHybridGiProvider))
            .with_priority(priority)
    }

    #[test]
    fn advanced_provider_selection_rejects_duplicate_virtual_geometry_ids() {
        let error = select_virtual_geometry_runtime_provider(vec![
            virtual_geometry_registration("same", 0),
            virtual_geometry_registration("same", 10),
        ])
        .expect_err("duplicate provider ids must be rejected");

        assert!(error
            .to_string()
            .contains("virtual_geometry provider `same` registered more than once"));
    }

    #[test]
    fn advanced_provider_selection_rejects_priority_ties() {
        let error = select_hybrid_gi_runtime_provider(vec![
            hybrid_gi_registration("first", 5),
            hybrid_gi_registration("second", 5),
        ])
        .expect_err("priority ties must be rejected");

        assert!(error
            .to_string()
            .contains("hybrid_global_illumination provider priority tie at 5"));
    }

    #[test]
    fn advanced_provider_selection_uses_highest_priority_and_reports_ids() {
        let virtual_geometry = select_virtual_geometry_runtime_provider(vec![
            virtual_geometry_registration("low", 1),
            virtual_geometry_registration("high", 20),
        ])
        .expect("virtual geometry provider selection should succeed")
        .expect("one virtual geometry provider should be selected");
        let hybrid_gi = select_hybrid_gi_runtime_provider(vec![hybrid_gi_registration("gi", 7)])
            .expect("hybrid gi provider selection should succeed")
            .expect("one hybrid gi provider should be selected");

        assert_eq!(virtual_geometry.provider_id(), "high");
        assert_eq!(hybrid_gi.provider_id(), "gi");

        let availability =
            selected_advanced_provider_availability(Some(&hybrid_gi), Some(&virtual_geometry));
        assert_eq!(
            availability.virtual_geometry_provider_id.as_deref(),
            Some("high")
        );
        assert_eq!(availability.hybrid_gi_provider_id.as_deref(), Some("gi"));
    }
}
