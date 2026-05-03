//! Graphics module-host registration absorbed into the runtime layer.

mod host;

use crate::engine_module::{EngineModule, ModuleDescriptor};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

pub use host::{
    module_descriptor, module_descriptor_with_render_features, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};

#[derive(Clone, Debug, Default)]
pub struct GraphicsModule {
    render_features: Vec<RenderFeatureDescriptor>,
    render_pass_executors: Vec<RenderPassExecutorRegistration>,
    hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
}

impl GraphicsModule {
    pub fn with_render_features(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
            render_pass_executors: Vec::new(),
            hybrid_gi_runtime_providers: Vec::new(),
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
            render_pass_executors: render_pass_executors.into_iter().collect(),
            hybrid_gi_runtime_providers: Vec::new(),
            virtual_geometry_runtime_providers: virtual_geometry_runtime_providers
                .into_iter()
                .collect(),
        }
    }

    pub fn with_render_extensions_and_runtime_providers(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
        hybrid_gi_runtime_providers: impl IntoIterator<Item = HybridGiRuntimeProviderRegistration>,
        virtual_geometry_runtime_providers: impl IntoIterator<
            Item = VirtualGeometryRuntimeProviderRegistration,
        >,
    ) -> Self {
        Self {
            render_features: render_features.into_iter().collect(),
            render_pass_executors: render_pass_executors.into_iter().collect(),
            hybrid_gi_runtime_providers: hybrid_gi_runtime_providers.into_iter().collect(),
            virtual_geometry_runtime_providers: virtual_geometry_runtime_providers
                .into_iter()
                .collect(),
        }
    }

    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        &self.render_pass_executors
    }

    pub fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        &self.hybrid_gi_runtime_providers
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
            self.render_pass_executors.clone(),
            self.hybrid_gi_runtime_providers.clone(),
            self.virtual_geometry_runtime_providers.clone(),
        )
    }
}
