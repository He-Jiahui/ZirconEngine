use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, VirtualGeometryRuntimeProviderRegistration,
};
use crate::rhi::RenderDevice;
use crate::rhi_wgpu::WgpuRenderDevice;

use crate::{GraphicsError, SceneRenderer};

use super::super::capability_summary::capability_summary;
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
        let render_features = render_features.into_iter().collect::<Vec<_>>();
        let render_pass_executors = render_pass_executors.into_iter().collect::<Vec<_>>();
        let runtime_prepare_collectors = runtime_prepare_collectors.into_iter().collect::<Vec<_>>();
        let hybrid_gi_runtime_providers =
            hybrid_gi_runtime_providers.into_iter().collect::<Vec<_>>();
        let virtual_geometry_runtime_providers = virtual_geometry_runtime_providers
            .into_iter()
            .collect::<Vec<_>>();
        let render_device = WgpuRenderDevice::new_headless();
        Ok(Self {
            state: Mutex::new(RenderFrameworkState {
                renderer: SceneRenderer::new_with_plugin_render_extensions(
                    asset_manager,
                    render_features.clone(),
                    render_pass_executors,
                    runtime_prepare_collectors,
                )?,
                next_viewport_id: 1,
                next_history_id: 1,
                pipelines: create_default_pipelines(&render_features),
                hybrid_gi_runtime_provider: hybrid_gi_runtime_providers.first().cloned(),
                virtual_geometry_runtime_provider: virtual_geometry_runtime_providers
                    .first()
                    .cloned(),
                last_virtual_geometry_debug_snapshot: None,
                viewports: HashMap::new(),
                stats: crate::core::framework::render::RenderStats {
                    capabilities: capability_summary(render_device.caps()),
                    ..crate::core::framework::render::RenderStats::default()
                },
            }),
        })
    }
}
