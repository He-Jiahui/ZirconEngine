use std::sync::Arc;

use crate::core::CoreHandle;
use crate::core::framework::render::{
    GeometrySourceDescriptor, RenderFramework, ShadingModelDescriptor,
};
use crate::graphics::{GraphicsError, WgpuRenderFramework};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

use crate::asset::{ProjectAssetManagerAccess, project_asset_manager_handle};

pub fn create_render_framework_with_render_features(
    core: &CoreHandle,
    render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
    plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
    render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
    solari_runtime_providers: impl IntoIterator<Item = SolariRuntimeProviderRegistration>,
    virtual_geometry_runtime_providers: impl IntoIterator<
        Item = VirtualGeometryRuntimeProviderRegistration,
    >,
) -> Result<Arc<dyn RenderFramework>, GraphicsError> {
    let asset_manager = ProjectAssetManagerAccess::new(
        core.clone(),
        project_asset_manager_handle(core)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?,
    );
    Ok(Arc::new(
        WgpuRenderFramework::new_with_plugin_render_extensions_and_solari_and_compute_task_pool(
            asset_manager,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            solari_runtime_providers,
            virtual_geometry_runtime_providers,
            plugin_geometry_sources,
            plugin_shading_models,
            core.task_pools().compute().clone(),
        )?,
    ))
}
