use crate::asset::AssetImporterRegistry;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::bridge::InterfaceExport;

use super::{PluginModuleId, RuntimeExtensionRegistry};

mod metadata;
mod runtime_core;
mod scene_hook;

impl RuntimeExtensionRegistry {
    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        self.render_features.values()
    }

    pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        self.render_pass_executors.values()
    }

    pub fn runtime_prepare_collectors(&self) -> &[RuntimePrepareCollectorRegistration] {
        self.runtime_prepare_collectors.values()
    }

    pub fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        self.hybrid_gi_runtime_providers.values()
    }

    pub fn solari_runtime_providers(&self) -> &[SolariRuntimeProviderRegistration] {
        self.solari_runtime_providers.values()
    }

    pub fn virtual_geometry_runtime_providers(
        &self,
    ) -> &[VirtualGeometryRuntimeProviderRegistration] {
        self.virtual_geometry_runtime_providers.values()
    }

    pub fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }

    pub(in crate::plugin) fn plugin_interfaces(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &InterfaceExport)> {
        self.plugin_interfaces
            .iter()
            .map(|(owner, _, export)| (owner, export))
    }
}
