use super::super::super::descriptors::{ModuleDescriptor, RegistryName};
use super::super::super::state::ServiceEntry;
use super::descriptor_entries::{
    prepare_driver_entry, prepare_manager_entry, prepare_plugin_entry,
};
use crate::core::CoreError;

pub(super) fn prepare_three_descriptor_service_entries(
    owner_module: &str,
    descriptor: &ModuleDescriptor,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> Result<[(RegistryName, ServiceEntry); 3], CoreError> {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 3);
    match (driver_count, manager_count, plugin_count) {
        (3, 0, 0) => {
            let [first, second, third] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires exactly three drivers");
            };
            Ok([
                prepare_driver_entry(owner_module, first)?,
                prepare_driver_entry(owner_module, second)?,
                prepare_driver_entry(owner_module, third)?,
            ])
        }
        (2, 1, 0) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires exactly two drivers");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires one manager");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_manager_entry(owner_module, manager)?,
            ])
        }
        (2, 0, 1) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires exactly two drivers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (1, 2, 0) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires one driver");
            };
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires exactly two managers");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
            ])
        }
        (1, 1, 1) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires one driver");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires one manager");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (1, 0, 2) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("three-service registration requires one driver");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires exactly two plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (0, 3, 0) => {
            let [first, second, third] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires exactly three managers");
            };
            Ok([
                prepare_manager_entry(owner_module, first)?,
                prepare_manager_entry(owner_module, second)?,
                prepare_manager_entry(owner_module, third)?,
            ])
        }
        (0, 2, 1) => {
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires exactly two managers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires one plugin");
            };
            Ok([
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (0, 1, 2) => {
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("three-service registration requires one manager");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires exactly two plugins");
            };
            Ok([
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (0, 0, 3) => {
            let [first, second, third] = descriptor.plugins.as_slice() else {
                unreachable!("three-service registration requires exactly three plugins");
            };
            Ok([
                prepare_plugin_entry(owner_module, first)?,
                prepare_plugin_entry(owner_module, second)?,
                prepare_plugin_entry(owner_module, third)?,
            ])
        }
        _ => unreachable!("three-service registration requires exactly three service descriptors"),
    }
}
