use super::super::super::descriptors::{ModuleDescriptor, RegistryName};
use super::super::super::state::ServiceEntry;
use super::descriptor_entries::{
    prepare_driver_entry, prepare_manager_entry, prepare_plugin_entry,
};
use crate::core::error::CoreError;

pub(super) fn prepare_five_descriptor_service_entries(
    owner_module: &str,
    descriptor: &ModuleDescriptor,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> Result<[(RegistryName, ServiceEntry); 5], CoreError> {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 5);
    match (driver_count, manager_count, plugin_count) {
        (5, 0, 0) => {
            let [first, second, third, fourth, fifth] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly five drivers");
            };
            Ok([
                prepare_driver_entry(owner_module, first)?,
                prepare_driver_entry(owner_module, second)?,
                prepare_driver_entry(owner_module, third)?,
                prepare_driver_entry(owner_module, fourth)?,
                prepare_driver_entry(owner_module, fifth)?,
            ])
        }
        (4, 1, 0) => {
            let [first_driver, second_driver, third_driver, fourth_driver] =
                descriptor.drivers.as_slice()
            else {
                unreachable!("five-service registration requires exactly four drivers");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires one manager");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_driver_entry(owner_module, third_driver)?,
                prepare_driver_entry(owner_module, fourth_driver)?,
                prepare_manager_entry(owner_module, manager)?,
            ])
        }
        (4, 0, 1) => {
            let [first_driver, second_driver, third_driver, fourth_driver] =
                descriptor.drivers.as_slice()
            else {
                unreachable!("five-service registration requires exactly four drivers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_driver_entry(owner_module, third_driver)?,
                prepare_driver_entry(owner_module, fourth_driver)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (3, 2, 0) => {
            let [first_driver, second_driver, third_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly three drivers");
            };
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires exactly two managers");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_driver_entry(owner_module, third_driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
            ])
        }
        (3, 1, 1) => {
            let [first_driver, second_driver, third_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly three drivers");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires one manager");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_driver_entry(owner_module, third_driver)?,
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (3, 0, 2) => {
            let [first_driver, second_driver, third_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly three drivers");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly two plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_driver_entry(owner_module, third_driver)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (2, 3, 0) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly two drivers");
            };
            let [first_manager, second_manager, third_manager] = descriptor.managers.as_slice()
            else {
                unreachable!("five-service registration requires exactly three managers");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_manager_entry(owner_module, third_manager)?,
            ])
        }
        (2, 2, 1) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly two drivers");
            };
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires exactly two managers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (2, 1, 2) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly two drivers");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires one manager");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly two plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (2, 0, 3) => {
            let [first_driver, second_driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires exactly two drivers");
            };
            let [first_plugin, second_plugin, third_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly three plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, first_driver)?,
                prepare_driver_entry(owner_module, second_driver)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
                prepare_plugin_entry(owner_module, third_plugin)?,
            ])
        }
        (1, 4, 0) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires one driver");
            };
            let [first_manager, second_manager, third_manager, fourth_manager] =
                descriptor.managers.as_slice()
            else {
                unreachable!("five-service registration requires exactly four managers");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_manager_entry(owner_module, third_manager)?,
                prepare_manager_entry(owner_module, fourth_manager)?,
            ])
        }
        (1, 3, 1) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires one driver");
            };
            let [first_manager, second_manager, third_manager] = descriptor.managers.as_slice()
            else {
                unreachable!("five-service registration requires exactly three managers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires one plugin");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_manager_entry(owner_module, third_manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (1, 2, 2) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires one driver");
            };
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires exactly two managers");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly two plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (1, 1, 3) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires one driver");
            };
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires one manager");
            };
            let [first_plugin, second_plugin, third_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly three plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
                prepare_plugin_entry(owner_module, third_plugin)?,
            ])
        }
        (1, 0, 4) => {
            let [driver] = descriptor.drivers.as_slice() else {
                unreachable!("five-service registration requires one driver");
            };
            let [first_plugin, second_plugin, third_plugin, fourth_plugin] =
                descriptor.plugins.as_slice()
            else {
                unreachable!("five-service registration requires exactly four plugins");
            };
            Ok([
                prepare_driver_entry(owner_module, driver)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
                prepare_plugin_entry(owner_module, third_plugin)?,
                prepare_plugin_entry(owner_module, fourth_plugin)?,
            ])
        }
        (0, 5, 0) => {
            let [first, second, third, fourth, fifth] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires exactly five managers");
            };
            Ok([
                prepare_manager_entry(owner_module, first)?,
                prepare_manager_entry(owner_module, second)?,
                prepare_manager_entry(owner_module, third)?,
                prepare_manager_entry(owner_module, fourth)?,
                prepare_manager_entry(owner_module, fifth)?,
            ])
        }
        (0, 4, 1) => {
            let [first_manager, second_manager, third_manager, fourth_manager] =
                descriptor.managers.as_slice()
            else {
                unreachable!("five-service registration requires exactly four managers");
            };
            let [plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires one plugin");
            };
            Ok([
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_manager_entry(owner_module, third_manager)?,
                prepare_manager_entry(owner_module, fourth_manager)?,
                prepare_plugin_entry(owner_module, plugin)?,
            ])
        }
        (0, 3, 2) => {
            let [first_manager, second_manager, third_manager] = descriptor.managers.as_slice()
            else {
                unreachable!("five-service registration requires exactly three managers");
            };
            let [first_plugin, second_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly two plugins");
            };
            Ok([
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_manager_entry(owner_module, third_manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
            ])
        }
        (0, 2, 3) => {
            let [first_manager, second_manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires exactly two managers");
            };
            let [first_plugin, second_plugin, third_plugin] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly three plugins");
            };
            Ok([
                prepare_manager_entry(owner_module, first_manager)?,
                prepare_manager_entry(owner_module, second_manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
                prepare_plugin_entry(owner_module, third_plugin)?,
            ])
        }
        (0, 1, 4) => {
            let [manager] = descriptor.managers.as_slice() else {
                unreachable!("five-service registration requires one manager");
            };
            let [first_plugin, second_plugin, third_plugin, fourth_plugin] =
                descriptor.plugins.as_slice()
            else {
                unreachable!("five-service registration requires exactly four plugins");
            };
            Ok([
                prepare_manager_entry(owner_module, manager)?,
                prepare_plugin_entry(owner_module, first_plugin)?,
                prepare_plugin_entry(owner_module, second_plugin)?,
                prepare_plugin_entry(owner_module, third_plugin)?,
                prepare_plugin_entry(owner_module, fourth_plugin)?,
            ])
        }
        (0, 0, 5) => {
            let [first, second, third, fourth, fifth] = descriptor.plugins.as_slice() else {
                unreachable!("five-service registration requires exactly five plugins");
            };
            Ok([
                prepare_plugin_entry(owner_module, first)?,
                prepare_plugin_entry(owner_module, second)?,
                prepare_plugin_entry(owner_module, third)?,
                prepare_plugin_entry(owner_module, fourth)?,
                prepare_plugin_entry(owner_module, fifth)?,
            ])
        }
        _ => unreachable!("five-service registration requires exactly five service descriptors"),
    }
}
