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
        let manager_name = descriptor.name.to_string();
        if self.managers.contains_key(&manager_name) {
            return Err(RuntimeExtensionRegistryError::DuplicateManager(
                descriptor.name.to_string(),
            ));
        }
        let owner = self.intern_runtime_owner(&plugin_id)?;
        self.managers
            .register(owner, manager_name, descriptor)
            .expect("manager duplicate was prechecked");
        Ok(())
    }

    pub fn register_module(
        &mut self,
        descriptor: ModuleDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_module_descriptor(&descriptor)?;
        if self.modules.contains_key(&descriptor.name) {
            return Err(RuntimeExtensionRegistryError::DuplicateModule(
                descriptor.name,
            ));
        }
        let owner = self.intern_plugin_module(format!("{}.runtime", descriptor.name))?;
        self.modules
            .register(owner, descriptor.name.clone(), descriptor)
            .expect("module duplicate was prechecked");
        Ok(())
    }
}
