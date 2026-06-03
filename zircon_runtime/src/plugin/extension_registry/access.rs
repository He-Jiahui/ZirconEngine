use crate::asset::AssetImporterRegistry;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

use super::RuntimeExtensionRegistry;

mod metadata;
mod runtime_core;
mod scene_hook;

impl RuntimeExtensionRegistry {
    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
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

    pub fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }
}
