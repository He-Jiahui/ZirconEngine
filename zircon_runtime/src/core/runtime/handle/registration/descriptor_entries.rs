use std::collections::HashSet;

use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

use super::super::super::descriptors::{
    DependencySpec, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, PluginDescriptor,
    RegistryName,
};
use super::super::super::state::{ServiceEntry, ServiceEntryFactory};
use super::entry::service_entry;
use super::validation::validate_service_descriptor;

pub(super) fn prepare_service_entry(
    owner_module: &str,
    kind: ServiceKind,
    name: RegistryName,
    startup_mode: StartupMode,
    dependencies: &[DependencySpec],
    factory: ServiceEntryFactory,
    pending_keys: &mut HashSet<RegistryName>,
    pending_services: &mut Vec<(RegistryName, ServiceEntry)>,
) -> Result<(), CoreError> {
    validate_service_descriptor(owner_module, kind, &name, dependencies)?;
    if !pending_keys.insert(name.clone()) {
        return Err(CoreError::DuplicateService(name.to_string()));
    }
    pending_services.push((name, service_entry(startup_mode, dependencies, factory)));
    Ok(())
}

pub(super) fn prepare_two_descriptor_service_entries(
    owner_module: &str,
    descriptor: &ModuleDescriptor,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> Result<[(RegistryName, ServiceEntry); 2], CoreError> {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 2);
    match (driver_count, manager_count, plugin_count) {
        (2, 0, 0) => {
            let [first, second] = descriptor.drivers.as_slice() else {
                unreachable!("two-service registration requires exactly two drivers");
            };
            Ok([
                prepare_driver_entry(owner_module, first)?,
                prepare_driver_entry(owner_module, second)?,
            ])
        }
        (1, 1, 0) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("two-service registration requires one driver");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("two-service registration requires one manager");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, manager)?,
            ])
        }
        (1, 0, 1) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("two-service registration requires one driver");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("two-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (0, 2, 0) => {
            let [first, second] = descriptor.managers.as_slice() else {
                unreachable!("two-service registration requires exactly two managers");
            };
            Ok([
                prepare_manager_entry(owner_module, first)?,
                prepare_manager_entry(owner_module, second)?,
            ])
        }
        (0, 1, 1) => {
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("two-service registration requires one manager");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("two-service registration requires one plugin");
            };
            Ok([
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (0, 0, 2) => {
            let [first, second] = descriptor.plugins.as_slice() else {
                unreachable!("two-service registration requires exactly two plugins");
            };
            Ok([
                prepare_plugin_entry(owner_module, first)?,
                prepare_plugin_entry(owner_module, second)?,
            ])
        }
        _ => unreachable!("two-service registration requires exactly two service descriptors"),
    }
}

pub(super) fn prepare_single_descriptor_service_entry(
    owner_module: &str,
    descriptor: &ModuleDescriptor,
) -> Result<(RegistryName, ServiceEntry), CoreError> {
    if let [driver] = descriptor.drivers.as_slice() {
        return prepare_driver_entry(owner_module, driver);
    }
    if let [manager] = descriptor.managers.as_slice() {
        return prepare_manager_entry(owner_module, manager);
    }
    let [plugin] = descriptor.plugins.as_slice() else {
        unreachable!("single-service registration requires exactly one service descriptor");
    };
    prepare_plugin_entry(owner_module, plugin)
}

pub(super) fn prepare_single_service_entry(
    owner_module: &str,
    kind: ServiceKind,
    name: RegistryName,
    startup_mode: StartupMode,
    dependencies: &[DependencySpec],
    factory: ServiceEntryFactory,
) -> Result<(RegistryName, ServiceEntry), CoreError> {
    validate_service_descriptor(owner_module, kind, &name, dependencies)?;
    Ok((name, service_entry(startup_mode, dependencies, factory)))
}

pub(super) fn prepare_driver_entry(
    owner_module: &str,
    descriptor: &DriverDescriptor,
) -> Result<(RegistryName, ServiceEntry), CoreError> {
    prepare_single_service_entry(
        owner_module,
        ServiceKind::Driver,
        descriptor.name.clone(),
        descriptor.startup_mode,
        &descriptor.dependencies,
        ServiceEntryFactory::Service(descriptor.factory.clone()),
    )
}

pub(super) fn prepare_manager_entry(
    owner_module: &str,
    descriptor: &ManagerDescriptor,
) -> Result<(RegistryName, ServiceEntry), CoreError> {
    prepare_single_service_entry(
        owner_module,
        ServiceKind::Manager,
        descriptor.name.clone(),
        descriptor.startup_mode,
        &descriptor.dependencies,
        ServiceEntryFactory::Service(descriptor.factory.clone()),
    )
}

pub(super) fn prepare_plugin_entry(
    owner_module: &str,
    descriptor: &PluginDescriptor,
) -> Result<(RegistryName, ServiceEntry), CoreError> {
    prepare_single_service_entry(
        owner_module,
        ServiceKind::Plugin,
        descriptor.name.clone(),
        descriptor.startup_mode,
        &descriptor.dependencies,
        ServiceEntryFactory::Plugin(descriptor.factory.clone()),
    )
}
