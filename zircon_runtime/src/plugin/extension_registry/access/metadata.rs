use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginModuleId, PluginOptionManifest,
    UiComponentDescriptor,
};

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn components(&self) -> &[ComponentTypeDescriptor] {
        self.components.values()
    }

    pub fn ui_components(&self) -> &[UiComponentDescriptor] {
        self.ui_components.values()
    }

    pub fn plugin_options(&self) -> &[PluginOptionManifest] {
        self.plugin_options.values()
    }

    pub fn plugin_event_catalogs(&self) -> &[PluginEventCatalogManifest] {
        self.plugin_event_catalogs.values()
    }

    pub fn geometry_sources(&self) -> &[GeometrySourceDescriptor] {
        self.geometry_sources.values()
    }

    pub(in crate::plugin) fn geometry_source_entries(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &GeometrySourceDescriptor)> {
        self.geometry_sources
            .iter()
            .map(|(owner, _, descriptor)| (owner, descriptor))
    }

    pub fn shading_models(&self) -> &[ShadingModelDescriptor] {
        self.shading_models.values()
    }

    pub(in crate::plugin) fn shading_model_entries(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &ShadingModelDescriptor)> {
        self.shading_models
            .iter()
            .map(|(owner, _, descriptor)| (owner, descriptor))
    }
}
