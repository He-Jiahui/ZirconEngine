use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginOptionManifest,
    UiComponentDescriptor,
};

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
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
}
