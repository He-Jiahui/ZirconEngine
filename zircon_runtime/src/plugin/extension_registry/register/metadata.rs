use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginOptionManifest,
    RuntimeExtensionRegistryError, UiComponentDescriptor,
};

use super::super::validation::{
    validate_component_type_descriptor, validate_plugin_event_catalog_manifest,
    validate_plugin_option_manifest, validate_ui_component_descriptor,
};
use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn register_component(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_component_type_descriptor(&descriptor)?;
        if self
            .components
            .iter()
            .any(|existing| existing.type_id == descriptor.type_id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateComponentType(
                descriptor.type_id,
            ));
        }
        self.components.push(descriptor);
        Ok(())
    }

    pub fn register_ui_component(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_ui_component_descriptor(&descriptor)?;
        if self
            .ui_components
            .iter()
            .any(|existing| existing.component_id == descriptor.component_id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateUiComponent(
                descriptor.component_id,
            ));
        }
        self.ui_components.push(descriptor);
        Ok(())
    }

    pub fn register_plugin_option(
        &mut self,
        descriptor: PluginOptionManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_option_manifest(&descriptor)?;
        if self
            .plugin_options
            .iter()
            .any(|existing| existing.key == descriptor.key)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginOption(
                descriptor.key,
            ));
        }
        self.plugin_options.push(descriptor);
        Ok(())
    }

    pub fn register_plugin_event_catalog(
        &mut self,
        descriptor: PluginEventCatalogManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_event_catalog_manifest(&descriptor)?;
        if self
            .plugin_event_catalogs
            .iter()
            .any(|existing| existing.namespace == descriptor.namespace)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEventCatalog(
                descriptor.namespace,
            ));
        }
        self.plugin_event_catalogs.push(descriptor);
        Ok(())
    }
}
