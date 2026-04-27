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
