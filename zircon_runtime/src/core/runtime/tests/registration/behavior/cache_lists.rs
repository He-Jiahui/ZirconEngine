use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::lifecycle::{ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn register_single_immediate_service_keeps_exact_cached_service_lists() {
    let runtime = CoreRuntime::new();
    let service_name =
        RegistryName::from_parts("SingleImmediateModule", ServiceKind::Driver, "ClockDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("SingleImmediateModule", "single immediate service").with_driver(
                DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("SingleImmediateModule")
        .expect("single-service module should be registered");
    assert_eq!(module.service_names.as_ref(), [service_name.clone()]);
    assert_eq!(
        module.startup_service_names.as_ref(),
        [service_name.clone()]
    );
    assert_eq!(module.shutdown_service_names.as_ref(), [service_name]);
    assert!(Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert!(Arc::ptr_eq(
        &module.service_names,
        &module.shutdown_service_names
    ));
}

#[test]
fn register_lazy_multi_service_keeps_empty_startup_cache() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("LazyMultiModule", ServiceKind::Driver, "ClockDriver");
    let manager_name =
        RegistryName::from_parts("LazyMultiModule", ServiceKind::Manager, "ClockManager");

    runtime
        .register_module(
            ModuleDescriptor::new("LazyMultiModule", "lazy multi-service module")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("LazyMultiModule")
        .expect("lazy multi-service module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [driver_name.clone(), manager_name.clone()]
    );
    assert!(module.startup_service_names.is_empty());
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [manager_name, driver_name]
    );
}

#[test]
fn register_exact_two_all_immediate_services_reuses_owner_cache_for_startup_cache() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("TwoImmediateModule", ServiceKind::Driver, "ClockDriver");
    let manager_name =
        RegistryName::from_parts("TwoImmediateModule", ServiceKind::Manager, "ClockManager");

    runtime
        .register_module(
            ModuleDescriptor::new("TwoImmediateModule", "two immediate services")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("TwoImmediateModule")
        .expect("two-immediate module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [driver_name.clone(), manager_name.clone()]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [driver_name.clone(), manager_name.clone()]
    );
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [manager_name, driver_name]
    );
    assert!(Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.shutdown_service_names
    ));
}

#[test]
fn register_same_kind_multi_service_reuses_owner_cache_for_shutdown_cache() {
    let runtime = CoreRuntime::new();
    let first_driver_name =
        RegistryName::from_parts("SameKindMultiModule", ServiceKind::Driver, "ClockDriver");
    let second_driver_name =
        RegistryName::from_parts("SameKindMultiModule", ServiceKind::Driver, "AudioDriver");
    let third_driver_name =
        RegistryName::from_parts("SameKindMultiModule", ServiceKind::Driver, "InputDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("SameKindMultiModule", "same-kind multi-service module")
                .with_driver(DriverDescriptor::new(
                    first_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    second_driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    third_driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 2 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("SameKindMultiModule")
        .expect("same-kind multi-service module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            first_driver_name.clone(),
            second_driver_name.clone(),
            third_driver_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [second_driver_name.clone()]
    );
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [first_driver_name, second_driver_name, third_driver_name]
    );
    assert!(Arc::ptr_eq(
        &module.service_names,
        &module.shutdown_service_names
    ));
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
}

#[test]
fn register_single_startup_multi_service_keeps_direct_startup_cache() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("SingleStartupModule", ServiceKind::Driver, "ClockDriver");
    let manager_name =
        RegistryName::from_parts("SingleStartupModule", ServiceKind::Manager, "ClockManager");
    let plugin_name =
        RegistryName::from_parts("SingleStartupModule", ServiceKind::Plugin, "RenderPlugin");

    runtime
        .register_module(
            ModuleDescriptor::new("SingleStartupModule", "single startup multi-service module")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("SingleStartupModule")
        .expect("single-startup multi-service module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            driver_name.clone(),
            manager_name.clone(),
            plugin_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [manager_name.clone()]
    );
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [plugin_name, manager_name, driver_name]
    );
}

#[test]
fn register_exact_three_mixed_kind_services_keep_direct_caches() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("ExactThreeMixedModule", ServiceKind::Driver, "ClockDriver");
    let manager_name = RegistryName::from_parts(
        "ExactThreeMixedModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let plugin_name =
        RegistryName::from_parts("ExactThreeMixedModule", ServiceKind::Plugin, "RenderPlugin");

    runtime
        .register_module(
            ModuleDescriptor::new("ExactThreeMixedModule", "exact three mixed services")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("ExactThreeMixedModule")
        .expect("exact-three mixed module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            driver_name.clone(),
            manager_name.clone(),
            plugin_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [driver_name.clone(), plugin_name.clone()]
    );
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [plugin_name, manager_name, driver_name]
    );
}

#[test]
fn register_all_immediate_multi_service_reuses_owner_cache_for_startup_cache() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("AllImmediateModule", ServiceKind::Driver, "ClockDriver");
    let manager_name =
        RegistryName::from_parts("AllImmediateModule", ServiceKind::Manager, "ClockManager");
    let plugin_name =
        RegistryName::from_parts("AllImmediateModule", ServiceKind::Plugin, "RenderPlugin");

    runtime
        .register_module(
            ModuleDescriptor::new("AllImmediateModule", "all immediate multi-service module")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("AllImmediateModule")
        .expect("all-immediate multi-service module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            driver_name.clone(),
            manager_name.clone(),
            plugin_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [
            driver_name.clone(),
            manager_name.clone(),
            plugin_name.clone()
        ]
    );
    assert!(Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [plugin_name, manager_name, driver_name]
    );
}

#[test]
fn register_exact_four_mixed_kind_services_keep_direct_caches() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("FourMixedStartupModule", ServiceKind::Driver, "ClockDriver");
    let first_manager_name = RegistryName::from_parts(
        "FourMixedStartupModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let second_manager_name = RegistryName::from_parts(
        "FourMixedStartupModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let plugin_name = RegistryName::from_parts(
        "FourMixedStartupModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("FourMixedStartupModule", "four mixed-startup services")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("FourMixedStartupModule")
        .expect("four-service mixed-startup module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            driver_name.clone(),
            first_manager_name.clone(),
            second_manager_name.clone(),
            plugin_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [first_manager_name.clone(), plugin_name.clone()]
    );
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [
            plugin_name,
            first_manager_name,
            second_manager_name,
            driver_name
        ]
    );
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.shutdown_service_names
    ));
}

#[test]
fn register_exact_five_mixed_kind_services_keep_direct_caches() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("FiveMixedStartupModule", ServiceKind::Driver, "ClockDriver");
    let first_manager_name = RegistryName::from_parts(
        "FiveMixedStartupModule",
        ServiceKind::Manager,
        "ClockManager",
    );
    let second_manager_name = RegistryName::from_parts(
        "FiveMixedStartupModule",
        ServiceKind::Manager,
        "AssetManager",
    );
    let first_plugin_name = RegistryName::from_parts(
        "FiveMixedStartupModule",
        ServiceKind::Plugin,
        "RenderPlugin",
    );
    let second_plugin_name = RegistryName::from_parts(
        "FiveMixedStartupModule",
        ServiceKind::Plugin,
        "TelemetryPlugin",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("FiveMixedStartupModule", "five mixed-startup services")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    first_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    second_manager_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    first_plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    second_plugin_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("FiveMixedStartupModule")
        .expect("five-service mixed-startup module should be registered");
    assert_eq!(
        module.service_names.as_ref(),
        [
            driver_name.clone(),
            first_manager_name.clone(),
            second_manager_name.clone(),
            first_plugin_name.clone(),
            second_plugin_name.clone()
        ]
    );
    assert_eq!(
        module.startup_service_names.as_ref(),
        [first_manager_name.clone(), first_plugin_name.clone()]
    );
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.startup_service_names
    ));
    assert_eq!(
        module.shutdown_service_names.as_ref(),
        [
            first_plugin_name,
            second_plugin_name,
            first_manager_name,
            second_manager_name,
            driver_name
        ]
    );
    assert!(!Arc::ptr_eq(
        &module.service_names,
        &module.shutdown_service_names
    ));
}
