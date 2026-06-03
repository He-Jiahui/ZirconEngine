use crate::core::{ManagerDescriptor, ModuleDescriptor};
use crate::plugin::RuntimeExtensionRegistryError;

use super::super::validation::{validate_manager_plugin_id, validate_module_descriptor};
use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn register_manager(
        &mut self,
        plugin_id: impl Into<String>,
        descriptor: ManagerDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let plugin_id = plugin_id.into();
        validate_manager_plugin_id(&plugin_id)?;
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
        validate_module_descriptor(&descriptor)?;
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
}
