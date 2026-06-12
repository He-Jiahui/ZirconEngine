use crate::asset::AssetImporterRegistry;
use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::bridge::InterfaceExport;
use crate::{
    plugin::ComponentTypeDescriptor, plugin::PluginEventCatalogManifest,
    plugin::PluginOptionManifest, plugin::SceneRuntimeHookRegistration,
    plugin::UiComponentDescriptor,
};
use std::any::TypeId;

use super::owner::{PluginModuleId, PluginModuleInterner};
use super::ownership::ExtensionOwnership;
use super::register::{
    EventRegistration, ResourceRegistration, RuntimeSceneSystemRegistration, SystemRegistration,
};
use super::typed_extension_point::TypedExtensionPoint;

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
    pub(super) render_features: TypedExtensionPoint<String, RenderFeatureDescriptor>,
    pub(super) render_pass_executors: TypedExtensionPoint<String, RenderPassExecutorRegistration>,
    pub(super) runtime_prepare_collectors:
        TypedExtensionPoint<String, RuntimePrepareCollectorRegistration>,
    pub(super) hybrid_gi_runtime_providers:
        TypedExtensionPoint<String, HybridGiRuntimeProviderRegistration>,
    pub(super) solari_runtime_providers:
        TypedExtensionPoint<String, SolariRuntimeProviderRegistration>,
    pub(super) virtual_geometry_runtime_providers:
        TypedExtensionPoint<String, VirtualGeometryRuntimeProviderRegistration>,
    pub(super) components: TypedExtensionPoint<String, ComponentTypeDescriptor>,
    pub(super) ui_components: TypedExtensionPoint<String, UiComponentDescriptor>,
    pub(super) plugin_options: TypedExtensionPoint<String, PluginOptionManifest>,
    pub(super) plugin_event_catalogs: TypedExtensionPoint<String, PluginEventCatalogManifest>,
    pub(super) asset_importers: AssetImporterRegistry,
    pub(super) scene_hooks: TypedExtensionPoint<String, SceneRuntimeHookRegistration>,
}

impl RuntimeExtensionRegistry {
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
            render_features: self.render_features.entries_owned_by(owner).collect(),
            render_pass_executors: self.render_pass_executors.entries_owned_by(owner).collect(),
            runtime_prepare_collectors: self
                .runtime_prepare_collectors
                .entries_owned_by(owner)
                .collect(),
            hybrid_gi_runtime_providers: self
                .hybrid_gi_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            solari_runtime_providers: self
                .solari_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            virtual_geometry_runtime_providers: self
                .virtual_geometry_runtime_providers
                .entries_owned_by(owner)
                .collect(),
            components: self.components.entries_owned_by(owner).collect(),
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

        ExtensionOwnership {
            plugin_systems: self.plugin_systems.remove_owned_by(owner),
            plugin_runtime_systems: self.plugin_runtime_systems.remove_owned_by(owner),
            plugin_resources: self.plugin_resources.remove_owned_by(owner),
            plugin_events: self.plugin_events.remove_owned_by(owner),
            plugin_interfaces: self.plugin_interfaces.remove_owned_by(owner),
            managers: self.managers.remove_owned_by(owner),
            modules: self.modules.remove_owned_by(owner),
            render_features: self.render_features.remove_owned_by(owner),
            render_pass_executors: self.render_pass_executors.remove_owned_by(owner),
            runtime_prepare_collectors: self.runtime_prepare_collectors.remove_owned_by(owner),
            hybrid_gi_runtime_providers: self.hybrid_gi_runtime_providers.remove_owned_by(owner),
            solari_runtime_providers: self.solari_runtime_providers.remove_owned_by(owner),
            virtual_geometry_runtime_providers: self
                .virtual_geometry_runtime_providers
                .remove_owned_by(owner),
            components: self.components.remove_owned_by(owner),
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
