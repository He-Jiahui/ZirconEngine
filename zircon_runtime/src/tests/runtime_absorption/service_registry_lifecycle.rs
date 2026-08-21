use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreResult, CoreRuntime, CoreWeak, ManagerDescriptor, ModuleContext,
    ModuleDescriptor, ModuleLifecycle, RegistryName, ServiceKind, StartupMode,
};

#[derive(Debug)]
struct WeakBackReferenceService {
    _core: CoreWeak,
}

#[derive(Debug)]
struct FailFinishLifecycle;

impl ModuleLifecycle for FailFinishLifecycle {
    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        Err(CoreError::MissingConfig(
            "tests.service_registry.finish".to_string(),
        ))
    }
}

#[test]
fn failed_service_initialization_does_not_retain_the_runtime_root() {
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();
    let service_name = RegistryName::from_parts(
        "WeakFailureModule",
        ServiceKind::Manager,
        "WeakFailureManager",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("WeakFailureModule", "weak failure lifecycle").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|core| {
                        let _construction_back_reference = core.clone();
                        Err(CoreError::MissingConfig(
                            "tests.service_registry.initialization".to_string(),
                        ))
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("WeakFailureModule").unwrap();

    assert!(
        runtime
            .resolve_manager::<WeakBackReferenceService>(service_name.as_str())
            .is_err()
    );
    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "failed service construction must release the Runtime root"
    );
}

#[test]
fn module_activation_rollback_drops_registry_owned_weak_services() {
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();
    let service_name = RegistryName::from_parts(
        "WeakRollbackModule",
        ServiceKind::Manager,
        "WeakRollbackManager",
    );

    runtime
        .register_module(
            ModuleDescriptor::new("WeakRollbackModule", "weak rollback lifecycle")
                .with_lifecycle(Arc::new(FailFinishLifecycle))
                .with_manager(ManagerDescriptor::new(
                    service_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|core| {
                        Ok(Arc::new(WeakBackReferenceService {
                            _core: core.clone(),
                        }) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    assert!(runtime.activate_module("WeakRollbackModule").is_err());
    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "module rollback must remove registry service instances before Runtime drop"
    );
}

#[test]
fn service_factory_panic_unwind_does_not_retain_the_runtime_root() {
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();
    let service_name =
        RegistryName::from_parts("WeakPanicModule", ServiceKind::Manager, "WeakPanicManager");

    runtime
        .register_module(
            ModuleDescriptor::new("WeakPanicModule", "weak panic lifecycle").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(|core| -> CoreResult<ServiceObject> {
                        let _service = WeakBackReferenceService {
                            _core: core.clone(),
                        };
                        panic!("deliberate service factory unwind")
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("WeakPanicModule").unwrap();

    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        let _ = runtime.resolve_manager::<WeakBackReferenceService>(service_name.as_str());
    }));
    assert!(panic_result.is_err());
    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "service factory unwind must release construction-only Runtime references"
    );
}
