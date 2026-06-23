use std::sync::Arc;

use crate::core::StartupMode;

use super::super::super::super::descriptors::RegistryName;
use super::super::super::super::state::ServiceEntry;
use super::shutdown::shutdown_service_names_or_owner_clone;
use super::types::ModuleServiceLists;

pub(super) struct MultiServiceListScan {
    pub(super) service_names: Arc<[RegistryName]>,
    pub(super) immediate_count: usize,
    pub(super) single_immediate_index: usize,
}

pub(super) fn scan_multi_service_module_lists(
    pending_services: &[(RegistryName, ServiceEntry)],
) -> MultiServiceListScan {
    debug_assert!(pending_services.len() >= 6);

    let mut service_names = Vec::with_capacity(pending_services.len());
    let mut immediate_count = 0_usize;
    let mut single_immediate_index = 0_usize;
    for (index, (name, entry)) in pending_services.iter().enumerate() {
        service_names.push(name.clone());
        if entry.startup_mode == StartupMode::Immediate {
            immediate_count += 1;
            if immediate_count == 1 {
                single_immediate_index = index;
            }
        }
    }

    MultiServiceListScan {
        service_names: service_names.into(),
        immediate_count,
        single_immediate_index,
    }
}

pub(super) fn single_startup_multi_service_module_lists(
    service_names: Arc<[RegistryName]>,
    pending_services: &[(RegistryName, ServiceEntry)],
    startup_service_index: usize,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(
        pending_services[startup_service_index].1.startup_mode,
        StartupMode::Immediate
    );
    let startup_service_name = pending_services[startup_service_index].0.clone();
    let shutdown_service_names = shutdown_service_names_or_owner_clone(
        &service_names,
        pending_services,
        driver_count,
        manager_count,
        plugin_count,
    );

    ModuleServiceLists {
        service_names,
        startup_service_names: Arc::<[RegistryName]>::from([startup_service_name]),
        shutdown_service_names,
    }
}

pub(super) fn mixed_startup_multi_service_module_lists(
    service_names: Arc<[RegistryName]>,
    pending_services: &[(RegistryName, ServiceEntry)],
    immediate_count: usize,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    let mut startup_service_names = Vec::with_capacity(immediate_count);
    for (name, entry) in pending_services {
        if entry.startup_mode == StartupMode::Immediate {
            startup_service_names.push(name.clone());
        }
    }

    let shutdown_service_names = shutdown_service_names_or_owner_clone(
        &service_names,
        pending_services,
        driver_count,
        manager_count,
        plugin_count,
    );

    ModuleServiceLists {
        service_names,
        startup_service_names: startup_service_names.into(),
        shutdown_service_names,
    }
}

pub(super) fn all_immediate_multi_service_module_lists(
    service_names: Arc<[RegistryName]>,
    pending_services: &[(RegistryName, ServiceEntry)],
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    let startup_service_names = service_names.clone();

    let shutdown_service_names = shutdown_service_names_or_owner_clone(
        &service_names,
        pending_services,
        driver_count,
        manager_count,
        plugin_count,
    );

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

pub(super) fn lazy_multi_service_module_lists(
    service_names: Arc<[RegistryName]>,
    pending_services: &[(RegistryName, ServiceEntry)],
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    let shutdown_service_names = shutdown_service_names_or_owner_clone(
        &service_names,
        pending_services,
        driver_count,
        manager_count,
        plugin_count,
    );

    ModuleServiceLists {
        service_names,
        startup_service_names: Arc::default(),
        shutdown_service_names,
    }
}
