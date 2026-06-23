use std::sync::Arc;

use super::super::super::super::descriptors::RegistryName;
use super::super::super::super::state::ServiceEntry;

pub(super) fn shutdown_service_names_or_owner_clone(
    owner_service_names: &Arc<[RegistryName]>,
    pending_services: &[(RegistryName, ServiceEntry)],
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> Arc<[RegistryName]> {
    if shutdown_order_matches_owner_order(
        pending_services.len(),
        driver_count,
        manager_count,
        plugin_count,
    ) {
        return owner_service_names.clone();
    }

    let mut shutdown_service_names = Vec::with_capacity(pending_services.len());
    push_shutdown_service_names(
        &mut shutdown_service_names,
        pending_services,
        driver_count,
        manager_count,
        plugin_count,
    );
    shutdown_service_names.into()
}

fn shutdown_order_matches_owner_order(
    service_count: usize,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> bool {
    driver_count == service_count || manager_count == service_count || plugin_count == service_count
}

fn push_shutdown_service_names(
    target: &mut Vec<RegistryName>,
    pending_services: &[(RegistryName, ServiceEntry)],
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) {
    let manager_start = driver_count;
    let plugin_start = driver_count + manager_count;
    let plugin_end = plugin_start + plugin_count;
    // Pending services are prepared in driver, manager, plugin descriptor order;
    // shutdown keeps the inverse plugin, manager, driver lifecycle order.
    push_service_names(target, &pending_services[plugin_start..plugin_end]);
    push_service_names(target, &pending_services[manager_start..plugin_start]);
    push_service_names(target, &pending_services[..driver_count]);
}

fn push_service_names(target: &mut Vec<RegistryName>, services: &[(RegistryName, ServiceEntry)]) {
    for (name, _) in services {
        target.push(name.clone());
    }
}
