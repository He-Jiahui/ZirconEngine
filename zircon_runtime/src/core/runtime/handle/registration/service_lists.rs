use std::sync::Arc;

use crate::core::StartupMode;

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;

pub(super) struct ModuleServiceLists {
    pub(super) service_names: Arc<[RegistryName]>,
    pub(super) startup_service_names: Arc<[RegistryName]>,
    pub(super) shutdown_service_names: Arc<[RegistryName]>,
}

pub(super) fn module_service_lists(
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

struct MultiServiceListScan {
    service_names: Arc<[RegistryName]>,
    immediate_count: usize,
    single_immediate_index: usize,
}

fn scan_multi_service_module_lists(
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

fn single_startup_multi_service_module_lists(
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

fn mixed_startup_multi_service_module_lists(
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

fn all_immediate_multi_service_module_lists(
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

fn lazy_multi_service_module_lists(
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

pub(super) fn single_service_module_lists(
    name: &RegistryName,
    entry: &ServiceEntry,
) -> ModuleServiceLists {
    let service_names = Arc::<[RegistryName]>::from([name.clone()]);
    let startup_service_names = if entry.startup_mode == StartupMode::Immediate {
        service_names.clone()
    } else {
        Arc::default()
    };
    let shutdown_service_names = service_names.clone();

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

fn two_service_module_lists(
    first_name: &RegistryName,
    first_entry: &ServiceEntry,
    second_name: &RegistryName,
    second_entry: &ServiceEntry,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 2);

    let service_names = Arc::<[RegistryName]>::from([first_name.clone(), second_name.clone()]);

    let first_immediate = first_entry.startup_mode == StartupMode::Immediate;
    let second_immediate = second_entry.startup_mode == StartupMode::Immediate;
    let startup_service_names = match (first_immediate, second_immediate) {
        (true, true) => service_names.clone(),
        (true, false) => Arc::<[RegistryName]>::from([first_name.clone()]),
        (false, true) => Arc::<[RegistryName]>::from([second_name.clone()]),
        (false, false) => Arc::default(),
    };

    let shutdown_service_names = if driver_count == 2 || manager_count == 2 || plugin_count == 2 {
        service_names.clone()
    } else {
        Arc::<[RegistryName]>::from([second_name.clone(), first_name.clone()])
    };

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

fn three_service_module_lists(
    first_name: &RegistryName,
    first_entry: &ServiceEntry,
    second_name: &RegistryName,
    second_entry: &ServiceEntry,
    third_name: &RegistryName,
    third_entry: &ServiceEntry,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 3);

    let service_names =
        Arc::<[RegistryName]>::from([first_name.clone(), second_name.clone(), third_name.clone()]);

    let first_immediate = first_entry.startup_mode == StartupMode::Immediate;
    let second_immediate = second_entry.startup_mode == StartupMode::Immediate;
    let third_immediate = third_entry.startup_mode == StartupMode::Immediate;
    let startup_service_names = match (first_immediate, second_immediate, third_immediate) {
        (true, true, true) => service_names.clone(),
        (true, true, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), second_name.clone()])
        }
        (true, false, true) => {
            Arc::<[RegistryName]>::from([first_name.clone(), third_name.clone()])
        }
        (false, true, true) => {
            Arc::<[RegistryName]>::from([second_name.clone(), third_name.clone()])
        }
        (true, false, false) => Arc::<[RegistryName]>::from([first_name.clone()]),
        (false, true, false) => Arc::<[RegistryName]>::from([second_name.clone()]),
        (false, false, true) => Arc::<[RegistryName]>::from([third_name.clone()]),
        (false, false, false) => Arc::default(),
    };

    let shutdown_service_names = match (driver_count, manager_count, plugin_count) {
        (3, 0, 0) | (0, 3, 0) | (0, 0, 3) => service_names.clone(),
        (2, 1, 0) | (2, 0, 1) | (0, 2, 1) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (1, 2, 0) | (1, 0, 2) | (0, 1, 2) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            first_name.clone(),
        ]),
        (1, 1, 1) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            second_name.clone(),
            first_name.clone(),
        ]),
        _ => unreachable!("three-service registration requires exactly three services"),
    };

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

fn four_service_module_lists(
    first_name: &RegistryName,
    first_entry: &ServiceEntry,
    second_name: &RegistryName,
    second_entry: &ServiceEntry,
    third_name: &RegistryName,
    third_entry: &ServiceEntry,
    fourth_name: &RegistryName,
    fourth_entry: &ServiceEntry,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 4);

    let service_names = Arc::<[RegistryName]>::from([
        first_name.clone(),
        second_name.clone(),
        third_name.clone(),
        fourth_name.clone(),
    ]);

    let first_immediate = first_entry.startup_mode == StartupMode::Immediate;
    let second_immediate = second_entry.startup_mode == StartupMode::Immediate;
    let third_immediate = third_entry.startup_mode == StartupMode::Immediate;
    let fourth_immediate = fourth_entry.startup_mode == StartupMode::Immediate;
    let startup_service_names = match (
        first_immediate,
        second_immediate,
        third_immediate,
        fourth_immediate,
    ) {
        (true, true, true, true) => service_names.clone(),
        (true, true, true, false) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
        ]),
        (true, true, false, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            fourth_name.clone(),
        ]),
        (true, false, true, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (false, true, true, true) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (true, true, false, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), second_name.clone()])
        }
        (true, false, true, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), third_name.clone()])
        }
        (true, false, false, true) => {
            Arc::<[RegistryName]>::from([first_name.clone(), fourth_name.clone()])
        }
        (false, true, true, false) => {
            Arc::<[RegistryName]>::from([second_name.clone(), third_name.clone()])
        }
        (false, true, false, true) => {
            Arc::<[RegistryName]>::from([second_name.clone(), fourth_name.clone()])
        }
        (false, false, true, true) => {
            Arc::<[RegistryName]>::from([third_name.clone(), fourth_name.clone()])
        }
        (true, false, false, false) => Arc::<[RegistryName]>::from([first_name.clone()]),
        (false, true, false, false) => Arc::<[RegistryName]>::from([second_name.clone()]),
        (false, false, true, false) => Arc::<[RegistryName]>::from([third_name.clone()]),
        (false, false, false, true) => Arc::<[RegistryName]>::from([fourth_name.clone()]),
        (false, false, false, false) => Arc::default(),
    };

    let shutdown_service_names = match (driver_count, manager_count, plugin_count) {
        (4, 0, 0) | (0, 4, 0) | (0, 0, 4) => service_names.clone(),
        (3, 1, 0) | (3, 0, 1) | (0, 3, 1) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
        ]),
        (1, 3, 0) | (1, 0, 3) | (0, 1, 3) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            first_name.clone(),
        ]),
        (2, 2, 0) | (2, 0, 2) | (0, 2, 2) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            fourth_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (2, 1, 1) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            third_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (1, 2, 1) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            second_name.clone(),
            third_name.clone(),
            first_name.clone(),
        ]),
        (1, 1, 2) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            fourth_name.clone(),
            second_name.clone(),
            first_name.clone(),
        ]),
        _ => unreachable!("four-service registration requires exactly four services"),
    };

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

fn five_service_module_lists(
    first_name: &RegistryName,
    first_entry: &ServiceEntry,
    second_name: &RegistryName,
    second_entry: &ServiceEntry,
    third_name: &RegistryName,
    third_entry: &ServiceEntry,
    fourth_name: &RegistryName,
    fourth_entry: &ServiceEntry,
    fifth_name: &RegistryName,
    fifth_entry: &ServiceEntry,
    driver_count: usize,
    manager_count: usize,
    plugin_count: usize,
) -> ModuleServiceLists {
    debug_assert_eq!(driver_count + manager_count + plugin_count, 5);

    let service_names = Arc::<[RegistryName]>::from([
        first_name.clone(),
        second_name.clone(),
        third_name.clone(),
        fourth_name.clone(),
        fifth_name.clone(),
    ]);

    let first_immediate = first_entry.startup_mode == StartupMode::Immediate;
    let second_immediate = second_entry.startup_mode == StartupMode::Immediate;
    let third_immediate = third_entry.startup_mode == StartupMode::Immediate;
    let fourth_immediate = fourth_entry.startup_mode == StartupMode::Immediate;
    let fifth_immediate = fifth_entry.startup_mode == StartupMode::Immediate;
    let startup_service_names = match (
        first_immediate,
        second_immediate,
        third_immediate,
        fourth_immediate,
        fifth_immediate,
    ) {
        (true, true, true, true, true) => service_names.clone(),
        (true, true, true, true, false) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (true, true, true, false, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
            fifth_name.clone(),
        ]),
        (true, true, false, true, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (true, false, true, true, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (false, true, true, true, true) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (true, true, true, false, false) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
        ]),
        (true, true, false, true, false) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            fourth_name.clone(),
        ]),
        (true, true, false, false, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            second_name.clone(),
            fifth_name.clone(),
        ]),
        (true, false, true, true, false) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (true, false, true, false, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            third_name.clone(),
            fifth_name.clone(),
        ]),
        (true, false, false, true, true) => Arc::<[RegistryName]>::from([
            first_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (false, true, true, true, false) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (false, true, true, false, true) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fifth_name.clone(),
        ]),
        (false, true, false, true, true) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (false, false, true, true, true) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
        ]),
        (true, true, false, false, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), second_name.clone()])
        }
        (true, false, true, false, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), third_name.clone()])
        }
        (true, false, false, true, false) => {
            Arc::<[RegistryName]>::from([first_name.clone(), fourth_name.clone()])
        }
        (true, false, false, false, true) => {
            Arc::<[RegistryName]>::from([first_name.clone(), fifth_name.clone()])
        }
        (false, true, true, false, false) => {
            Arc::<[RegistryName]>::from([second_name.clone(), third_name.clone()])
        }
        (false, true, false, true, false) => {
            Arc::<[RegistryName]>::from([second_name.clone(), fourth_name.clone()])
        }
        (false, true, false, false, true) => {
            Arc::<[RegistryName]>::from([second_name.clone(), fifth_name.clone()])
        }
        (false, false, true, true, false) => {
            Arc::<[RegistryName]>::from([third_name.clone(), fourth_name.clone()])
        }
        (false, false, true, false, true) => {
            Arc::<[RegistryName]>::from([third_name.clone(), fifth_name.clone()])
        }
        (false, false, false, true, true) => {
            Arc::<[RegistryName]>::from([fourth_name.clone(), fifth_name.clone()])
        }
        (true, false, false, false, false) => Arc::<[RegistryName]>::from([first_name.clone()]),
        (false, true, false, false, false) => Arc::<[RegistryName]>::from([second_name.clone()]),
        (false, false, true, false, false) => Arc::<[RegistryName]>::from([third_name.clone()]),
        (false, false, false, true, false) => Arc::<[RegistryName]>::from([fourth_name.clone()]),
        (false, false, false, false, true) => Arc::<[RegistryName]>::from([fifth_name.clone()]),
        (false, false, false, false, false) => Arc::default(),
    };

    let shutdown_service_names = match (driver_count, manager_count, plugin_count) {
        (5, 0, 0) | (0, 5, 0) | (0, 0, 5) => service_names.clone(),
        (4, 1, 0) | (4, 0, 1) | (0, 4, 1) => Arc::<[RegistryName]>::from([
            fifth_name.clone(),
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
        ]),
        (1, 4, 0) | (1, 0, 4) | (0, 1, 4) => Arc::<[RegistryName]>::from([
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
            first_name.clone(),
        ]),
        (3, 2, 0) | (3, 0, 2) | (0, 3, 2) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            fifth_name.clone(),
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
        ]),
        (2, 3, 0) | (2, 0, 3) | (0, 2, 3) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (3, 1, 1) => Arc::<[RegistryName]>::from([
            fifth_name.clone(),
            fourth_name.clone(),
            first_name.clone(),
            second_name.clone(),
            third_name.clone(),
        ]),
        (1, 3, 1) => Arc::<[RegistryName]>::from([
            fifth_name.clone(),
            second_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            first_name.clone(),
        ]),
        (1, 1, 3) => Arc::<[RegistryName]>::from([
            third_name.clone(),
            fourth_name.clone(),
            fifth_name.clone(),
            second_name.clone(),
            first_name.clone(),
        ]),
        (2, 2, 1) => Arc::<[RegistryName]>::from([
            fifth_name.clone(),
            third_name.clone(),
            fourth_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (2, 1, 2) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            fifth_name.clone(),
            third_name.clone(),
            first_name.clone(),
            second_name.clone(),
        ]),
        (1, 2, 2) => Arc::<[RegistryName]>::from([
            fourth_name.clone(),
            fifth_name.clone(),
            second_name.clone(),
            third_name.clone(),
            first_name.clone(),
        ]),
        _ => unreachable!("five-service registration requires exactly five services"),
    };

    ModuleServiceLists {
        service_names,
        startup_service_names,
        shutdown_service_names,
    }
}

fn shutdown_service_names_or_owner_clone(
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
