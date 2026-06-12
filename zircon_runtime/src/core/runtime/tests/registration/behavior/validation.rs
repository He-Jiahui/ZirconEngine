use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

#[test]
fn register_module_rejects_noncanonical_module_names() {
    for invalid in ["", " TestModule", "TestModule ", "\tTestModule"] {
        let runtime = CoreRuntime::new();
        let error = runtime
            .register_module(ModuleDescriptor::new(invalid, "invalid module name"))
            .unwrap_err();

        assert!(matches!(
            error,
            CoreError::InvalidModuleName(value) if value == invalid
        ));
    }
}

#[test]
fn register_module_rejects_descriptor_registry_kind_mismatch() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("MismatchModule", "mismatched registry kind").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts("MismatchModule", ServiceKind::Manager, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceKindMismatch {
            name,
            expected: ServiceKind::Driver,
            actual: ServiceKind::Manager,
        } if name == "MismatchModule.Manager.ClockDriver"
    ));
}

#[test]
fn register_module_rejects_descriptor_registry_owner_mismatch() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("OwnerModule", "mismatched registry owner").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts("OtherModule", ServiceKind::Driver, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceOwnerMismatch {
            name,
            expected,
            actual,
        } if name == "OtherModule.Driver.ClockDriver"
            && expected == "OwnerModule"
            && actual == "OtherModule"
    ));
}

#[test]
fn register_module_rejects_driver_dependencies_on_managers() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("DependencyModule", "invalid driver dependency")
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts(
                        "DependencyModule",
                        ServiceKind::Driver,
                        "ClockDependency",
                    ),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts(
                        "DependencyModule",
                        ServiceKind::Driver,
                        "ClockDriver",
                    ),
                    StartupMode::Immediate,
                    vec![
                        DependencySpec::named(RegistryName::from_parts(
                            "DependencyModule",
                            ServiceKind::Driver,
                            "ClockDependency",
                        )),
                        DependencySpec::named(RegistryName::from_parts(
                            "DependencyModule",
                            ServiceKind::Manager,
                            "ClockManager",
                        )),
                    ],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::InvalidServiceDependencyKind {
            service,
            service_kind: ServiceKind::Driver,
            dependency,
            dependency_kind: ServiceKind::Manager,
        } if service == "DependencyModule.Driver.ClockDriver"
            && dependency == "DependencyModule.Manager.ClockManager"
    ));
}

#[test]
fn register_module_rejects_third_driver_dependency_on_manager() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new(
                "ThreeDependencyValidationModule",
                "invalid third dependency",
            )
            .with_driver(DriverDescriptor::new(
                RegistryName::from_parts(
                    "ThreeDependencyValidationModule",
                    ServiceKind::Driver,
                    "ClockDriver",
                ),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(RegistryName::from_parts(
                        "ThreeDependencyValidationModule",
                        ServiceKind::Driver,
                        "ClockDependency",
                    )),
                    DependencySpec::named(RegistryName::from_parts(
                        "ThreeDependencyValidationModule",
                        ServiceKind::Driver,
                        "AudioDependency",
                    )),
                    DependencySpec::named(RegistryName::from_parts(
                        "ThreeDependencyValidationModule",
                        ServiceKind::Manager,
                        "BadManagerDependency",
                    )),
                ],
                Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
            )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::InvalidServiceDependencyKind {
            service,
            service_kind: ServiceKind::Driver,
            dependency,
            dependency_kind: ServiceKind::Manager,
        } if service == "ThreeDependencyValidationModule.Driver.ClockDriver"
            && dependency == "ThreeDependencyValidationModule.Manager.BadManagerDependency"
    ));
}

#[test]
fn register_module_rejects_fourth_driver_dependency_on_manager() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new(
                "FourDependencyValidationModule",
                "invalid fourth dependency",
            )
            .with_driver(DriverDescriptor::new(
                RegistryName::from_parts(
                    "FourDependencyValidationModule",
                    ServiceKind::Driver,
                    "ClockDriver",
                ),
                StartupMode::Immediate,
                vec![
                    DependencySpec::named(RegistryName::from_parts(
                        "FourDependencyValidationModule",
                        ServiceKind::Driver,
                        "ClockDependency",
                    )),
                    DependencySpec::named(RegistryName::from_parts(
                        "FourDependencyValidationModule",
                        ServiceKind::Driver,
                        "AudioDependency",
                    )),
                    DependencySpec::named(RegistryName::from_parts(
                        "FourDependencyValidationModule",
                        ServiceKind::Driver,
                        "InputDependency",
                    )),
                    DependencySpec::named(RegistryName::from_parts(
                        "FourDependencyValidationModule",
                        ServiceKind::Manager,
                        "BadManagerDependency",
                    )),
                ],
                Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
            )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::InvalidServiceDependencyKind {
            service,
            service_kind: ServiceKind::Driver,
            dependency,
            dependency_kind: ServiceKind::Manager,
        } if service == "FourDependencyValidationModule.Driver.ClockDriver"
            && dependency == "FourDependencyValidationModule.Manager.BadManagerDependency"
    ));
}

#[test]
fn register_module_rejects_fifth_driver_dependency_on_manager() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("FiveDependencyValidationModule", "invalid fifth dependency")
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts(
                        "FiveDependencyValidationModule",
                        ServiceKind::Driver,
                        "ClockDriver",
                    ),
                    StartupMode::Immediate,
                    vec![
                        DependencySpec::named(RegistryName::from_parts(
                            "FiveDependencyValidationModule",
                            ServiceKind::Driver,
                            "ClockDependency",
                        )),
                        DependencySpec::named(RegistryName::from_parts(
                            "FiveDependencyValidationModule",
                            ServiceKind::Driver,
                            "AudioDependency",
                        )),
                        DependencySpec::named(RegistryName::from_parts(
                            "FiveDependencyValidationModule",
                            ServiceKind::Driver,
                            "InputDependency",
                        )),
                        DependencySpec::named(RegistryName::from_parts(
                            "FiveDependencyValidationModule",
                            ServiceKind::Driver,
                            "RenderDependency",
                        )),
                        DependencySpec::named(RegistryName::from_parts(
                            "FiveDependencyValidationModule",
                            ServiceKind::Manager,
                            "BadManagerDependency",
                        )),
                    ],
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::InvalidServiceDependencyKind {
            service,
            service_kind: ServiceKind::Driver,
            dependency,
            dependency_kind: ServiceKind::Manager,
        } if service == "FiveDependencyValidationModule.Driver.ClockDriver"
            && dependency == "FiveDependencyValidationModule.Manager.BadManagerDependency"
    ));
}

#[test]
fn register_module_does_not_leave_services_when_descriptor_validation_fails() {
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(
            ModuleDescriptor::new("RollbackModule", "failed descriptor registration")
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts("RollbackModule", ServiceKind::Driver, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts("RollbackModule", ServiceKind::Driver, "BadManager"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::ServiceKindMismatch {
            name,
            expected: ServiceKind::Manager,
            actual: ServiceKind::Driver,
        } if name == "RollbackModule.Driver.BadManager"
    ));
    let missing_driver = runtime
        .resolve_driver::<TestDriver>("RollbackModule.Driver.ClockDriver")
        .unwrap_err();
    assert!(matches!(
        missing_driver,
        CoreError::MissingService(name) if name == "RollbackModule.Driver.ClockDriver"
    ));
}

#[test]
fn register_empty_module_skips_pending_service_preparation() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new("EmptyRegistrationModule", "empty"))
        .unwrap();

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("EmptyRegistrationModule")
        .expect("empty module should be registered");
    assert!(module.service_names.is_empty());
    assert!(module.startup_service_names.is_empty());
    assert!(module.shutdown_service_names.is_empty());
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    assert!(services.is_empty());
    drop(services);

    let duplicate = runtime
        .register_module(ModuleDescriptor::new(
            "EmptyRegistrationModule",
            "duplicate",
        ))
        .unwrap_err();
    assert!(matches!(
        duplicate,
        CoreError::DuplicateModule(name) if name == "EmptyRegistrationModule"
    ));
}
