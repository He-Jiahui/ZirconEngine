use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind, StartupMode};

#[test]
fn deactivate_single_service_module_reports_external_dependent() {
    let runtime = CoreRuntime::new();
    let driver_name =
        RegistryName::from_parts("SingleOwnerModule", ServiceKind::Driver, "ClockDriver");
    let dependent_manager_name = RegistryName::from_parts(
        "SingleDependentModule",
        ServiceKind::Manager,
        "ClockManager",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("SingleOwnerModule", "single owner").with_driver(
                DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SingleDependentModule", "single dependent").with_manager(
                ManagerDescriptor::new(
                    dependent_manager_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name.clone())],
                    Arc::new(|core| {
                        let _ = core
                            .resolve_driver::<TestDriver>("SingleOwnerModule.Driver.ClockDriver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();

    runtime.activate_module("SingleOwnerModule").unwrap();
    runtime.activate_module("SingleDependentModule").unwrap();

    let error = runtime.deactivate_module("SingleOwnerModule").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected single-service unload to report the running external dependent");
    };
    assert_eq!(blocked_service, driver_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);

    let handle = runtime.handle();
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(driver_name.as_str())
            .expect("blocked single-service deactivate should keep the owner service");
        assert_eq!(entry.lifecycle, LifecycleState::Running);
        assert!(entry.instance.is_some());
    }

    runtime.deactivate_module("SingleDependentModule").unwrap();
    runtime.deactivate_module("SingleOwnerModule").unwrap();

    let services = handle.inner.services.lock().unwrap();
    let entry = services
        .get(driver_name.as_str())
        .expect("successful single-service deactivate should keep the owner service entry");
    assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
    assert!(entry.instance.is_none());
}

#[test]
fn deactivate_blocks_when_external_dependents_are_alive() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts("ModuleA", ServiceKind::Driver, "ClockDriver");
    let local_manager_name =
        RegistryName::from_parts("ModuleA", ServiceKind::Manager, "LocalManager");
    let dependent_manager_name =
        RegistryName::from_parts("ModuleB", ServiceKind::Manager, "ClockManager");

    runtime
        .register_module(
            ModuleDescriptor::new("ModuleA", "a")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    local_manager_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(ModuleDescriptor::new("ModuleB", "b").with_manager(
            ManagerDescriptor::new(
                dependent_manager_name.clone(),
                StartupMode::Immediate,
                vec![DependencySpec::named(driver_name.clone())],
                Arc::new(|core| {
                    let _ = core.resolve_driver::<TestDriver>("ModuleA.Driver.ClockDriver")?;
                    Ok(Arc::new(TestManager) as ServiceObject)
                }),
            ),
        ))
        .unwrap();

    runtime.activate_module("ModuleA").unwrap();
    runtime.activate_module("ModuleB").unwrap();
    let error = runtime.deactivate_module("ModuleA").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected unload block when a running external dependent is alive");
    };
    assert_eq!(blocked_service, driver_name.as_str());
    assert_eq!(dependents, vec![dependent_manager_name.to_string()]);

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("ModuleA")
            .expect("blocked deactivate should keep module registered");
        assert_eq!(module.lifecycle, LifecycleState::Running);
    }
    {
        let services = handle.inner.services.lock().unwrap();
        for service_name in [&driver_name, &local_manager_name] {
            let entry = services
                .get(service_name.as_str())
                .expect("blocked deactivate should keep owner service registered");
            assert_eq!(entry.lifecycle, LifecycleState::Running);
            assert!(entry.instance.is_some());
        }
    }

    runtime.deactivate_module("ModuleB").unwrap();
    runtime.deactivate_module("ModuleA").unwrap();

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&driver_name, &local_manager_name] {
        let entry = services
            .get(service_name.as_str())
            .expect("successful deactivate should keep owner service entry");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
    }
}
