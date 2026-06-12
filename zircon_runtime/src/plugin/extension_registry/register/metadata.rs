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
        if self.components.contains_key(&descriptor.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateComponentType(
                descriptor.type_id,
            ));
        }
        let owner = self.intern_runtime_owner(&descriptor.plugin_id)?;
        self.components
            .register(owner, descriptor.type_id.clone(), descriptor)
            .expect("component duplicate was prechecked");
        Ok(())
    }

    pub fn register_ui_component(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_ui_component_descriptor(&descriptor)?;
        if self.ui_components.contains_key(&descriptor.component_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateUiComponent(
                descriptor.component_id,
            ));
        }
        let owner = self.intern_runtime_owner(&descriptor.plugin_id)?;
        self.ui_components
            .register(owner, descriptor.component_id.clone(), descriptor)
            .expect("ui component duplicate was prechecked");
        Ok(())
    }

    pub fn register_plugin_option(
        &mut self,
        descriptor: PluginOptionManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_option_manifest(&descriptor)?;
        if self.plugin_options.contains_key(&descriptor.key) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginOption(
                descriptor.key,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&descriptor.key)?;
        self.plugin_options
            .register(owner, descriptor.key.clone(), descriptor)
            .expect("plugin option duplicate was prechecked");
        Ok(())
    }

    pub fn register_plugin_event_catalog(
        &mut self,
        descriptor: PluginEventCatalogManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_event_catalog_manifest(&descriptor)?;
        if self
            .plugin_event_catalogs
            .contains_key(&descriptor.namespace)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEventCatalog(
                descriptor.namespace,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&descriptor.namespace)?;
        self.plugin_event_catalogs
            .register(owner, descriptor.namespace.clone(), descriptor)
            .expect("plugin event catalog duplicate was prechecked");
        Ok(())
    }
}
