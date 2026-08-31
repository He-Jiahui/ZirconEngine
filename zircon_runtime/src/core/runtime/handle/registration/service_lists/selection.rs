use super::multi::{
    all_immediate_multi_service_module_lists, lazy_multi_service_module_lists,
    mixed_startup_multi_service_module_lists, scan_multi_service_module_lists,
    single_startup_multi_service_module_lists,
};
use super::specialized::{
    five_service_module_lists, four_service_module_lists, single_service_module_lists,
    three_service_module_lists, two_service_module_lists,
};
use super::types::ModuleServiceLists;
use crate::core::runtime::descriptors::RegistryName;
use crate::core::runtime::state::ServiceEntry;

pub(in crate::core::runtime::handle::registration) fn module_service_lists(
    pending_services: &[(RegistryName, ServiceEntry)],
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(
        pending_services.len(),
        driver_count + manager_count + plugin_count
    );
    if let [(name, entry)] = pending_services {
        return single_service_module_lists(name, entry);
    }
    if let [(first_name, first_entry), (second_name, second_entry)] = pending_services {
        return two_service_module_lists(
            first_name,
            first_entry,
            second_name,
            second_entry,
            driver_count,
            manager_count,
            plugin_count,
        );
    }
    if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry)] =
        pending_services
    {
        return three_service_module_lists(
            first_name,
            first_entry,
            second_name,
            second_entry,
            third_name,
            third_entry,
            driver_count,
            manager_count,
            plugin_count,
        );
    }
    if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry)] =
        pending_services
    {
        return four_service_module_lists(
            first_name,
            first_entry,
            second_name,
            second_entry,
            third_name,
            third_entry,
            fourth_name,
            fourth_entry,
            driver_count,
            manager_count,
            plugin_count,
        );
    }
    if let [(first_name, first_entry), (second_name, second_entry), (third_name, third_entry), (fourth_name, fourth_entry), (fifth_name, fifth_entry)] =
        pending_services
    {
        return five_service_module_lists(
            first_name,
            first_entry,
            second_name,
            second_entry,
            third_name,
            third_entry,
            fourth_name,
            fourth_entry,
            fifth_name,
            fifth_entry,
            driver_count,
            manager_count,
            plugin_count,
        );
    }

    let scan = scan_multi_service_module_lists(pending_services);
    if scan.immediate_count == 0 {
        return lazy_multi_service_module_lists(
            scan.service_names,
            pending_services,
            driver_count,
            manager_count,
            plugin_count,
        );
    }
    if scan.immediate_count == pending_services.len() {
        return all_immediate_multi_service_module_lists(
            scan.service_names,
            pending_services,
            driver_count,
            manager_count,
            plugin_count,
        );
    }
    if scan.immediate_count == 1 {
        return single_startup_multi_service_module_lists(
            scan.service_names,
            pending_services,
            scan.single_immediate_index,
            driver_count,
            manager_count,
            plugin_count,
        );
    }

    mixed_startup_multi_service_module_lists(
        scan.service_names,
        pending_services,
        scan.immediate_count,
        driver_count,
        manager_count,
        plugin_count,
    )
}
