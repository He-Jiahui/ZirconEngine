use std::collections::HashSet;

use crate::asset::AssetImporterRegistry;
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::bridge::InterfaceExport;
use crate::plugin::PluginShaderModuleSource;

use super::{PluginModuleId, RuntimeExtensionRegistry};

mod metadata;
mod runtime_core;

impl RuntimeExtensionRegistry {
    pub fn shader_module_sources(&self) -> &[PluginShaderModuleSource] {
        &self.shader_module_sources
    }

    #[cfg(feature = "graphics")]
    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        self.render_features.values()
    }

    #[cfg(feature = "graphics")]
    pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        self.render_pass_executors.values()
    }

    #[cfg(feature = "graphics")]
    pub fn runtime_prepare_collectors(&self) -> &[RuntimePrepareCollectorRegistration] {
        self.runtime_prepare_collectors.values()
    }

    #[cfg(feature = "graphics")]
    pub fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        self.hybrid_gi_runtime_providers.values()
    }

    #[cfg(feature = "graphics")]
    pub fn solari_runtime_providers(&self) -> &[SolariRuntimeProviderRegistration] {
        self.solari_runtime_providers.values()
    }

    #[cfg(feature = "graphics")]
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

    pub(crate) fn interface_exports_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> impl Iterator<Item = (&str, &InterfaceExport)> {
        self.plugin_interfaces
            .iter()
            .filter_map(move |(candidate, interface_id, export)| {
                (candidate == owner).then_some((interface_id.as_str(), export))
            })
    }

    pub fn interface_owners_for_runtime_modules<I, S>(&self, module_names: I) -> Vec<PluginModuleId>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let module_names = module_names
            .into_iter()
            .map(|module_name| module_name.as_ref().to_string())
            .collect::<HashSet<_>>();
        sorted_unique_interface_owners(self.plugin_interfaces.iter().filter_map(|(owner, _, _)| {
            let module_name = self.plugin_module_name(owner)?;
            module_names.contains(module_name).then_some(owner)
        }))
    }
}

fn sorted_unique_interface_owners(
    owners: impl IntoIterator<Item = PluginModuleId>,
) -> Vec<PluginModuleId> {
    let mut unique = HashSet::new();
    for owner in owners {
        unique.insert(owner);
    }
    let mut owners = unique.into_iter().collect::<Vec<_>>();
    owners.sort_by_key(|owner| owner.raw());
    owners
}

#[cfg(test)]
#[path = "access/interface_owner_dedup_tests.rs"]
mod interface_owner_dedup_tests;
