//! Graphics module-host registration absorbed into the runtime layer.

mod host;

use crate::core::framework::render::{
    GeometrySourceDescriptor, ShadingModelDescriptor, GRAPHICS_MODULE_NAME,
};
use crate::engine_module::{EngineModule, ModuleDescriptor};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

pub use host::{
    module_descriptor, module_descriptor_with_render_features, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME,
};

#[derive(Clone, Debug, Default)]
pub struct GraphicsModule {
    render_features: Vec<RenderFeatureDescriptor>,
    plugin_geometry_sources: Vec<GeometrySourceDescriptor>,
    plugin_shading_models: Vec<ShadingModelDescriptor>,
    render_pass_executors: Vec<RenderPassExecutorRegistration>,
    runtime_prepare_collectors: Vec<RuntimePrepareCollectorRegistration>,
    hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    solari_runtime_providers: Vec<SolariRuntimeProviderRegistration>,
    virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
}

impl GraphicsModule {
    pub fn with_render_features(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
            plugin_geometry_sources: Vec::new(),
            plugin_shading_models: Vec::new(),
            render_pass_executors: Vec::new(),
            runtime_prepare_collectors: Vec::new(),
            hybrid_gi_runtime_providers: Vec::new(),
            solari_runtime_providers: Vec::new(),
            virtual_geometry_runtime_providers: Vec::new(),
        }
    }

    pub fn with_render_extensions(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
            plugin_geometry_sources: Vec::new(),
            plugin_shading_models: Vec::new(),
            render_pass_executors: render_pass_executors.into_iter().collect(),
            runtime_prepare_collectors: Vec::new(),
            hybrid_gi_runtime_providers: Vec::new(),
            solari_runtime_providers: Vec::new(),
            virtual_geometry_runtime_providers: virtual_geometry_runtime_providers
                .into_iter()
                .collect(),
        }
    }

    pub fn with_render_extensions_and_runtime_providers(
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
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
            plugin_geometry_sources: plugin_geometry_sources.into_iter().collect(),
            plugin_shading_models: plugin_shading_models.into_iter().collect(),
            render_pass_executors: render_pass_executors.into_iter().collect(),
            runtime_prepare_collectors: runtime_prepare_collectors.into_iter().collect(),
            hybrid_gi_runtime_providers: hybrid_gi_runtime_providers.into_iter().collect(),
            solari_runtime_providers: solari_runtime_providers.into_iter().collect(),
            virtual_geometry_runtime_providers: virtual_geometry_runtime_providers
                .into_iter()
                .collect(),
        }
    }

    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    pub fn plugin_geometry_sources(&self) -> &[GeometrySourceDescriptor] {
        &self.plugin_geometry_sources
    }

    pub fn plugin_shading_models(&self) -> &[ShadingModelDescriptor] {
        &self.plugin_shading_models
    }

    pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        &self.render_pass_executors
    }

    pub fn runtime_prepare_collectors(&self) -> &[RuntimePrepareCollectorRegistration] {
        &self.runtime_prepare_collectors
    }

    pub fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        &self.hybrid_gi_runtime_providers
    }

    pub fn solari_runtime_providers(&self) -> &[SolariRuntimeProviderRegistration] {
        &self.solari_runtime_providers
    }

    pub fn virtual_geometry_runtime_providers(
        &self,
    ) -> &[VirtualGeometryRuntimeProviderRegistration] {
        &self.virtual_geometry_runtime_providers
    }
}

impl EngineModule for GraphicsModule {
    fn module_name(&self) -> &'static str {
        GRAPHICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Rendering device abstraction and scene rendering"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor_with_render_features(
            self.render_features.clone(),
            self.plugin_geometry_sources.clone(),
            self.plugin_shading_models.clone(),
            self.render_pass_executors.clone(),
            self.runtime_prepare_collectors.clone(),
            self.hybrid_gi_runtime_providers.clone(),
            self.solari_runtime_providers.clone(),
            self.virtual_geometry_runtime_providers.clone(),
        )
    }
}
