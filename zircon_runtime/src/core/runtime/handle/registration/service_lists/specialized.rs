use std::sync::Arc;

use crate::core::StartupMode;

use super::super::super::super::descriptors::RegistryName;
use super::super::super::super::state::ServiceEntry;
use super::types::ModuleServiceLists;

pub(in crate::core::runtime::handle::registration) fn single_service_module_lists(
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

pub(super) fn two_service_module_lists(
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

pub(super) fn three_service_module_lists(
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

pub(super) fn four_service_module_lists(
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

pub(super) fn five_service_module_lists(
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
