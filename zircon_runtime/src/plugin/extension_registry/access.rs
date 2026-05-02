use crate::asset::AssetImporterRegistry;
use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::{
    plugin::ComponentTypeDescriptor, plugin::PluginEventCatalogManifest, plugin::PluginOptionManifest,
    plugin::UiComponentDescriptor,
};

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn managers(&self) -> &[ManagerDescriptor] {
        &self.managers
    }

    pub fn modules(&self) -> &[ModuleDescriptor] {
        &self.modules
    }

    pub fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        &self.render_pass_executors
    }

    pub fn virtual_geometry_runtime_providers(
        &self,
    ) -> &[VirtualGeometryRuntimeProviderRegistration] {
        &self.virtual_geometry_runtime_providers
    }

    pub fn components(&self) -> &[ComponentTypeDescriptor] {
        &self.components
    }

    pub fn ui_components(&self) -> &[UiComponentDescriptor] {
        &self.ui_components
    }

    pub fn plugin_options(&self) -> &[PluginOptionManifest] {
        &self.plugin_options
    }

    pub fn plugin_event_catalogs(&self) -> &[PluginEventCatalogManifest] {
        &self.plugin_event_catalogs
    }

    pub fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }
}
