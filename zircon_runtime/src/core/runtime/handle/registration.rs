use std::collections::HashMap;

use crate::core::error::CoreError;
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};

use super::super::descriptors::{DependencySpec, ModuleDescriptor, RegistryName};
use super::super::state::{ModuleEntry, ServiceEntry, ServiceEntryFactory};
use super::CoreHandle;

impl CoreHandle {
    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        let module_name = descriptor.name.clone();
        let mut modules = self.inner.modules.lock().unwrap();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }

        {
            let mut services = self.inner.services.lock().unwrap();
            for driver in &descriptor.drivers {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Driver,
                    driver.name.clone(),
                    driver.startup_mode,
                    driver.dependencies.clone(),
                    ServiceEntryFactory::Service(driver.factory.clone()),
                )?;
            }
            for manager in &descriptor.managers {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Manager,
                    manager.name.clone(),
                    manager.startup_mode,
                    manager.dependencies.clone(),
                    ServiceEntryFactory::Service(manager.factory.clone()),
                )?;
            }
            for plugin in &descriptor.plugins {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Plugin,
                    plugin.name.clone(),
                    plugin.startup_mode,
                    plugin.dependencies.clone(),
                    ServiceEntryFactory::Plugin(plugin.factory.clone()),
                )?;
            }
        }

        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn insert_service(
        &self,
        services: &mut HashMap<String, ServiceEntry>,
        owner_module: String,
        kind: ServiceKind,
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceEntryFactory,
    ) -> Result<(), CoreError> {
        let key = name.to_string();
        if services.contains_key(&key) {
            return Err(CoreError::DuplicateService(key));
        }
        services.insert(
            key,
            ServiceEntry {
                name,
                owner_module,
                kind,
                startup_mode,
                dependencies,
                factory,
                lifecycle: LifecycleState::Registered,
                instance: None,
            },
        );
        Ok(())
    }
}
