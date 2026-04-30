use crate::core::{ManagerDescriptor, ModuleDescriptor};
use crate::graphics::RenderFeatureDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, RuntimeExtensionRegistryError, UiComponentDescriptor,
};

use super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn register_manager(
        &mut self,
        _plugin_id: impl Into<String>,
        descriptor: ManagerDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .managers
            .iter()
            .any(|existing| existing.name == descriptor.name)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateManager(
                descriptor.name.to_string(),
            ));
        }
        self.managers.push(descriptor);
        Ok(())
    }

    pub fn register_module(
        &mut self,
        descriptor: ModuleDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .modules
            .iter()
            .any(|existing| existing.name == descriptor.name)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateModule(
                descriptor.name,
            ));
        }
        self.modules.push(descriptor);
        Ok(())
    }

    pub fn register_render_feature(
        &mut self,
        descriptor: RenderFeatureDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .render_features
            .iter()
            .any(|existing| existing.name == descriptor.name)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateRenderFeature(
                descriptor.name,
            ));
        }
        self.render_features.push(descriptor);
        Ok(())
    }

    pub fn register_component(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let expected_prefix = format!("{}.", descriptor.plugin_id);
        if !descriptor.type_id.starts_with(&expected_prefix) {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!(
                    "component type {} must be prefixed by plugin id {}",
                    descriptor.type_id, descriptor.plugin_id
                ),
            ));
        }
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
        let expected_prefix = format!("{}.", descriptor.plugin_id);
        if !descriptor.component_id.starts_with(&expected_prefix) {
            return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
                "ui component {} must be prefixed by plugin id {}",
                descriptor.component_id, descriptor.plugin_id
            )));
        }
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
}
