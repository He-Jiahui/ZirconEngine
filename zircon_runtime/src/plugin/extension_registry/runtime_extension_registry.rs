use crate::asset::AssetImporterRegistry;
#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::bridge::InterfaceExport;
#[cfg(feature = "ui")]
use crate::plugin::UiComponentDescriptor;
use crate::{
    plugin::ComponentTypeDescriptor, plugin::PluginEventCatalogManifest,
    plugin::PluginOptionManifest, plugin::SceneRuntimeHookRegistration,
};
use std::any::TypeId;

use super::owner::{PluginModuleId, PluginModuleInterner};
use super::ownership::ExtensionOwnership;
use super::register::{
    EventRegistration, ResourceRegistration, RuntimeSceneSystemRegistration, SystemRegistration,
};
use super::typed_extension_point::TypedExtensionPoint;

#[cfg(test)]
#[path = "runtime_extension_registry/tests.rs"]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct RuntimeExtensionRegistry {
    pub(super) plugin_modules: PluginModuleInterner,
    pub(super) system_sets: crate::scene::ecs::SystemSetRegistry,
    pub(super) plugin_systems: TypedExtensionPoint<String, SystemRegistration>,
    pub(super) plugin_runtime_systems: TypedExtensionPoint<String, RuntimeSceneSystemRegistration>,
    pub(super) plugin_resources: TypedExtensionPoint<TypeId, ResourceRegistration>,
    pub(super) plugin_events: TypedExtensionPoint<TypeId, EventRegistration>,
    pub(super) plugin_interfaces: TypedExtensionPoint<String, InterfaceExport>,
    pub(super) managers: TypedExtensionPoint<String, ManagerDescriptor>,
    pub(super) modules: TypedExtensionPoint<String, ModuleDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) render_features: TypedExtensionPoint<String, RenderFeatureDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) render_pass_executors: TypedExtensionPoint<String, RenderPassExecutorRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) geometry_sources: TypedExtensionPoint<String, GeometrySourceDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) shading_models: TypedExtensionPoint<String, ShadingModelDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) runtime_prepare_collectors:
        TypedExtensionPoint<String, RuntimePrepareCollectorRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) hybrid_gi_runtime_providers:
        TypedExtensionPoint<String, HybridGiRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) solari_runtime_providers:
        TypedExtensionPoint<String, SolariRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) virtual_geometry_runtime_providers:
        TypedExtensionPoint<String, VirtualGeometryRuntimeProviderRegistration>,
    pub(super) components: TypedExtensionPoint<String, ComponentTypeDescriptor>,
    #[cfg(feature = "ui")]
    pub(super) ui_components: TypedExtensionPoint<String, UiComponentDescriptor>,
    pub(super) plugin_options: TypedExtensionPoint<String, PluginOptionManifest>,
    pub(super) plugin_event_catalogs: TypedExtensionPoint<String, PluginEventCatalogManifest>,
    pub(super) asset_importers: AssetImporterRegistry,
    pub(super) asset_importers_finalized: bool,
    pub(super) scene_hooks: TypedExtensionPoint<String, SceneRuntimeHookRegistration>,
}

impl RuntimeExtensionRegistry {
    /// Finalizes every extension family after catalog validation and merge.
    /// Owner reload, registration, or revocation may invalidate the epoch and
    /// then finalize the registry again.
    pub fn finalize(&mut self) {
        self.plugin_systems.freeze();
        self.plugin_runtime_systems.freeze();
        self.plugin_resources.freeze();
        self.plugin_events.freeze();
        self.plugin_interfaces.freeze();
        self.managers.freeze();
        self.modules.freeze();
        #[cfg(feature = "graphics")]
        self.render_features.freeze();
        #[cfg(feature = "graphics")]
        self.render_pass_executors.freeze();
        #[cfg(feature = "graphics")]
        self.geometry_sources.freeze();
        #[cfg(feature = "graphics")]
        self.shading_models.freeze();
        #[cfg(feature = "graphics")]
        self.runtime_prepare_collectors.freeze();
        #[cfg(feature = "graphics")]
        self.hybrid_gi_runtime_providers.freeze();
        #[cfg(feature = "graphics")]
        self.solari_runtime_providers.freeze();
        #[cfg(feature = "graphics")]
        self.virtual_geometry_runtime_providers.freeze();
        self.components.freeze();
        #[cfg(feature = "ui")]
        self.ui_components.freeze();
        self.plugin_options.freeze();
        self.plugin_event_catalogs.freeze();
        self.asset_importers_finalized = true;
        self.scene_hooks.freeze();
    }

    pub fn is_finalized(&self) -> bool {
        let finalized = self.plugin_systems.is_frozen()
            && self.plugin_runtime_systems.is_frozen()
            && self.plugin_resources.is_frozen()
            && self.plugin_events.is_frozen()
            && self.plugin_interfaces.is_frozen()
            && self.managers.is_frozen()
            && self.modules.is_frozen();
        #[cfg(feature = "graphics")]
        let finalized = finalized
            && self.render_features.is_frozen()
            && self.render_pass_executors.is_frozen()
            && self.geometry_sources.is_frozen()
            && self.shading_models.is_frozen()
            && self.runtime_prepare_collectors.is_frozen()
            && self.hybrid_gi_runtime_providers.is_frozen()
            && self.solari_runtime_providers.is_frozen()
            && self.virtual_geometry_runtime_providers.is_frozen();
        let finalized = finalized
            && self.components.is_frozen()
            && self.plugin_options.is_frozen()
            && self.plugin_event_catalogs.is_frozen()
            && self.asset_importers_finalized
            && self.scene_hooks.is_frozen();
        #[cfg(feature = "ui")]
        let finalized = finalized && self.ui_components.is_frozen();
        finalized
    }

    pub fn ownership_for(&self, owner: PluginModuleId) -> ExtensionOwnership {
        let asset_importers = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_id_from_module_name)
            .map(|plugin_id| self.asset_importers.descriptors_for_plugin(plugin_id))
            .unwrap_or_default();

        ExtensionOwnership {
            plugin_systems: self.plugin_systems.entries_owned_by(owner).collect(),
            plugin_runtime_systems: self
                .plugin_runtime_systems
                .entries_owned_by(owner)
                .collect(),
            plugin_resources: self.plugin_resources.entries_owned_by(owner).collect(),
            plugin_events: self.plugin_events.entries_owned_by(owner).collect(),
            plugin_interfaces: self.plugin_interfaces.entries_owned_by(owner).collect(),
            managers: self.managers.entries_owned_by(owner).collect(),
            modules: self.modules.entries_owned_by(owner).collect(),
            #[cfg(feature = "graphics")]
            render_features: self.render_features.entries_owned_by(owner).collect(),
            #[cfg(feature = "graphics")]
            render_pass_executors: self.render_pass_executors.entries_owned_by(owner).collect(),
            #[cfg(feature = "graphics")]
            geometry_sources: self.geometry_sources.entries_owned_by(owner).collect(),
            #[cfg(feature = "graphics")]
            shading_models: self.shading_models.entries_owned_by(owner).collect(),
            #[cfg(feature = "graphics")]
            runtime_prepare_collectors: self
                .runtime_prepare_collectors
                .entries_owned_by(owner)
                .collect(),
            #[cfg(feature = "graphics")]
            hybrid_gi_runtime_providers: self
                .hybrid_gi_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            #[cfg(feature = "graphics")]
            solari_runtime_providers: self
                .solari_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            #[cfg(feature = "graphics")]
            virtual_geometry_runtime_providers: self
                .virtual_geometry_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            components: self.components.entries_owned_by(owner).collect(),
            #[cfg(feature = "ui")]
            ui_components: self.ui_components.entries_owned_by(owner).collect(),
            plugin_options: self.plugin_options.entries_owned_by(owner).collect(),
            plugin_event_catalogs: self.plugin_event_catalogs.entries_owned_by(owner).collect(),
            asset_importers,
            scene_hooks: self.scene_hooks.entries_owned_by(owner).collect(),
        }
    }

    pub fn revoke_owner_registrations(&mut self, owner: PluginModuleId) -> ExtensionOwnership {
        let plugin_id = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_id_from_module_name)
            .map(str::to_owned);
        let asset_importers = plugin_id
            .as_deref()
            .map(|plugin_id| self.asset_importers.remove_by_plugin_id(plugin_id))
            .unwrap_or_default();
        if !asset_importers.is_empty() {
            self.asset_importers_finalized = false;
        }

        ExtensionOwnership {
            plugin_systems: self.plugin_systems.remove_owned_by(owner),
            plugin_runtime_systems: self.plugin_runtime_systems.remove_owned_by(owner),
            plugin_resources: self.plugin_resources.remove_owned_by(owner),
            plugin_events: self.plugin_events.remove_owned_by(owner),
            plugin_interfaces: self.plugin_interfaces.remove_owned_by(owner),
            managers: self.managers.remove_owned_by(owner),
            modules: self.modules.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            render_features: self.render_features.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            render_pass_executors: self.render_pass_executors.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            geometry_sources: self.geometry_sources.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            shading_models: self.shading_models.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            runtime_prepare_collectors: self.runtime_prepare_collectors.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            hybrid_gi_runtime_providers: self.hybrid_gi_runtime_providers.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            solari_runtime_providers: self.solari_runtime_providers.remove_owned_by(owner),
            #[cfg(feature = "graphics")]
            virtual_geometry_runtime_providers: self
                .virtual_geometry_runtime_providers
                .remove_owned_by(owner),
            components: self.components.remove_owned_by(owner),
            #[cfg(feature = "ui")]
            ui_components: self.ui_components.remove_owned_by(owner),
            plugin_options: self.plugin_options.remove_owned_by(owner),
            plugin_event_catalogs: self.plugin_event_catalogs.remove_owned_by(owner),
            asset_importers,
            scene_hooks: self.scene_hooks.remove_owned_by(owner),
        }
    }
}

fn plugin_id_from_module_name(module_name: &str) -> Option<&str> {
    module_name.strip_suffix(".runtime")
}
