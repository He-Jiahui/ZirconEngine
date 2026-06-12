use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginOptionManifest,
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
}
