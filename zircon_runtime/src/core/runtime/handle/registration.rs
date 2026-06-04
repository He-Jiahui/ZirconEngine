use std::collections::{HashMap, HashSet};

use crate::core::error::CoreError;
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};

use super::super::descriptors::{DependencySpec, ModuleDescriptor, RegistryName};
use super::super::state::{ModuleEntry, ServiceEntry, ServiceEntryFactory};
use super::CoreHandle;

impl CoreHandle {
    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "register_module");
        let module_name = descriptor.name.clone();
        if !is_canonical_module_name(&module_name) {
            return Err(CoreError::InvalidModuleName(module_name));
        }
        let mut modules = self.inner.modules.lock().unwrap();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }

        {
            let mut services = self.inner.services.lock().unwrap();
            let mut pending_services = Vec::with_capacity(
                descriptor.drivers.len() + descriptor.managers.len() + descriptor.plugins.len(),
            );
            let mut pending_keys = HashSet::new();
            for driver in &descriptor.drivers {
                self.prepare_service_entry(
                    module_name.clone(),
                    ServiceKind::Driver,
                    driver.name.clone(),
                    driver.startup_mode,
                    driver.dependencies.clone(),
                    ServiceEntryFactory::Service(driver.factory.clone()),
                    &services,
                    &mut pending_keys,
                    &mut pending_services,
                )?;
            }
            for manager in &descriptor.managers {
                self.prepare_service_entry(
                    module_name.clone(),
                    ServiceKind::Manager,
                    manager.name.clone(),
                    manager.startup_mode,
                    manager.dependencies.clone(),
                    ServiceEntryFactory::Service(manager.factory.clone()),
                    &services,
                    &mut pending_keys,
                    &mut pending_services,
                )?;
            }
            for plugin in &descriptor.plugins {
                self.prepare_service_entry(
                    module_name.clone(),
                    ServiceKind::Plugin,
                    plugin.name.clone(),
                    plugin.startup_mode,
                    plugin.dependencies.clone(),
                    ServiceEntryFactory::Plugin(plugin.factory.clone()),
                    &services,
                    &mut pending_keys,
                    &mut pending_services,
                )?;
            }
            for (key, entry) in pending_services {
                services.insert(key, entry);
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

    fn prepare_service_entry(
        &self,
        owner_module: String,
        kind: ServiceKind,
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceEntryFactory,
        existing_services: &HashMap<RegistryName, ServiceEntry>,
        pending_keys: &mut HashSet<RegistryName>,
        pending_services: &mut Vec<(RegistryName, ServiceEntry)>,
    ) -> Result<(), CoreError> {
        let key = name.clone();
        let key_text = name.to_string();
        let actual_owner = name.module_name();
        if actual_owner != owner_module {
            return Err(CoreError::ServiceOwnerMismatch {
                name: key_text,
                expected: owner_module.clone(),
                actual: actual_owner.to_string(),
            });
        }
        let actual_kind = name.service_kind();
        if actual_kind != kind {
            return Err(CoreError::ServiceKindMismatch {
                name: key_text,
                expected: kind,
                actual: actual_kind,
            });
        }
        validate_driver_dependencies(kind, &name, &dependencies)?;
        if existing_services.contains_key(name.as_str()) || !pending_keys.insert(key.clone()) {
            return Err(CoreError::DuplicateService(name.to_string()));
        }
        pending_services.push((
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
        ));
        Ok(())
    }
}

fn is_canonical_module_name(name: &str) -> bool {
    !name.is_empty() && name.trim() == name
}

fn validate_driver_dependencies(
    kind: ServiceKind,
    name: &RegistryName,
    dependencies: &[DependencySpec],
) -> Result<(), CoreError> {
    if kind != ServiceKind::Driver {
        return Ok(());
    }
    for dependency in dependencies {
        let dependency_kind = dependency.name.service_kind();
        if dependency_kind != ServiceKind::Driver {
            return Err(CoreError::InvalidServiceDependencyKind {
                service: name.to_string(),
                service_kind: kind,
                dependency: dependency.name.to_string(),
                dependency_kind,
            });
        }
    }
    Ok(())
}
