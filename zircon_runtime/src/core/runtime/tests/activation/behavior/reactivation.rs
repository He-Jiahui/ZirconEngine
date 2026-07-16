use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::TestManager;
use crate::core::framework::error::CoreResult;
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, LifecycleState, ServiceKind, StartupMode};

#[test]
fn single_module_reactivation_restores_immediate_and_lazy_service_slots() {
    assert_successful_reactivation(false);
}

#[test]
fn batch_module_reactivation_restores_immediate_and_lazy_service_slots() {
    assert_successful_reactivation(true);
}

#[test]
fn failed_reactivation_restores_unloaded_slots_and_invalidates_discarded_instance() {
    let runtime = CoreRuntime::new();
    let immediate_name = manager_name("ReactivationRollbackModule", "ImmediateManager");
    let lazy_name = manager_name("ReactivationRollbackModule", "LazyManager");
    let immediate_calls = Arc::new(AtomicUsize::new(0));
    let lazy_calls = Arc::new(AtomicUsize::new(0));
    let lifecycle = Arc::new(ArmableFinishFailure::default());

    runtime
        .register_module(
            ModuleDescriptor::new("ReactivationRollbackModule", "reactivation rollback")
                .with_lifecycle(lifecycle.clone())
                .with_manager(counted_manager(
                    immediate_name.clone(),
                    StartupMode::Immediate,
                    Arc::clone(&immediate_calls),
                ))
                .with_manager(counted_manager(
                    lazy_name.clone(),
                    StartupMode::Lazy,
                    Arc::clone(&lazy_calls),
                )),
        )
        .unwrap();
    runtime
        .activate_module("ReactivationRollbackModule")
        .unwrap();

    let handle = runtime.handle();
    let first_immediate_identity = handle
        .registered_manager_identity(immediate_name.as_str())
        .unwrap();
    let first_lazy_identity = handle
        .registered_manager_identity(lazy_name.as_str())
        .unwrap();
    runtime
        .deactivate_module("ReactivationRollbackModule")
        .unwrap();

    lifecycle.fail_next_finish();
    let error = runtime
        .activate_module("ReactivationRollbackModule")
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::MissingConfig(key) if key == "reactivation.finish"
    ));
    assert_eq!(immediate_calls.load(Ordering::SeqCst), 2);
    assert_eq!(lazy_calls.load(Ordering::SeqCst), 0);

    {
        let modules = handle.inner.modules.lock().unwrap();
        let module = modules
            .get("ReactivationRollbackModule")
            .expect("failed reactivation should keep the module entry");
        assert_eq!(module.lifecycle, LifecycleState::Unloaded);
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let immediate = services
            .get(&immediate_name)
            .expect("failed reactivation should keep the immediate slot");
        assert_eq!(immediate.lifecycle, LifecycleState::Unloaded);
        assert!(immediate.instance.is_none());
        assert_eq!(immediate.index, first_immediate_identity.index());
        assert_eq!(
            immediate.generation,
            first_immediate_identity.generation() + 2
        );

        let lazy = services
            .get(&lazy_name)
            .expect("failed reactivation should keep the lazy slot");
        assert_eq!(lazy.lifecycle, LifecycleState::Unloaded);
        assert!(lazy.instance.is_none());
        assert_eq!(lazy.index, first_lazy_identity.index());
        assert_eq!(lazy.generation, first_lazy_identity.generation() + 1);
    }

    runtime
        .activate_module("ReactivationRollbackModule")
        .unwrap();
    let current_immediate_identity = handle
        .registered_manager_identity(immediate_name.as_str())
        .unwrap();
    let current_lazy_identity = handle
        .registered_manager_identity(lazy_name.as_str())
        .unwrap();
    assert_eq!(
        current_immediate_identity.generation(),
        first_immediate_identity.generation() + 2
    );
    assert_eq!(
        current_lazy_identity.generation(),
        first_lazy_identity.generation() + 1
    );
    assert_eq!(immediate_calls.load(Ordering::SeqCst), 3);
    assert_eq!(lazy_calls.load(Ordering::SeqCst), 0);
}

fn assert_successful_reactivation(batch: bool) {
    let runtime = CoreRuntime::new();
    let module_name = if batch {
        "BatchReactivationModule"
    } else {
        "SingleReactivationModule"
    };
    let immediate_name = manager_name(module_name, "ImmediateManager");
    let lazy_name = manager_name(module_name, "LazyManager");
    let immediate_calls = Arc::new(AtomicUsize::new(0));
    let lazy_calls = Arc::new(AtomicUsize::new(0));

    runtime
        .register_module(
            ModuleDescriptor::new(module_name, "service slot reactivation")
                .with_manager(counted_manager(
                    immediate_name.clone(),
                    StartupMode::Immediate,
                    Arc::clone(&immediate_calls),
                ))
                .with_manager(counted_manager(
                    lazy_name.clone(),
                    StartupMode::Lazy,
                    Arc::clone(&lazy_calls),
                )),
        )
        .unwrap();
    activate(&runtime, module_name, batch);

    let handle = runtime.handle();
    let first_immediate_identity = handle
        .registered_manager_identity(immediate_name.as_str())
        .unwrap();
    let first_lazy_identity = handle
        .registered_manager_identity(lazy_name.as_str())
        .unwrap();
    assert_eq!(immediate_calls.load(Ordering::SeqCst), 1);
    assert_eq!(lazy_calls.load(Ordering::SeqCst), 0);

    runtime.deactivate_module(module_name).unwrap();
    activate(&runtime, module_name, batch);

    let current_immediate_identity = handle
        .registered_manager_identity(immediate_name.as_str())
        .unwrap();
    let current_lazy_identity = handle
        .registered_manager_identity(lazy_name.as_str())
        .unwrap();
    assert_eq!(
        current_immediate_identity.index(),
        first_immediate_identity.index()
    );
    assert_eq!(current_lazy_identity.index(), first_lazy_identity.index());
    assert_eq!(
        current_immediate_identity.generation(),
        first_immediate_identity.generation() + 1
    );
    assert_eq!(
        current_lazy_identity.generation(),
        first_lazy_identity.generation() + 1
    );
    assert_eq!(immediate_calls.load(Ordering::SeqCst), 2);
    assert_eq!(lazy_calls.load(Ordering::SeqCst), 0);

    {
        let services = handle.inner.services.lock().unwrap();
        let immediate = services
            .get(&immediate_name)
            .expect("reactivation should keep the immediate slot");
        assert_eq!(immediate.lifecycle, LifecycleState::Running);
        assert!(immediate.instance.is_some());

        let lazy = services
            .get(&lazy_name)
            .expect("reactivation should keep the lazy slot");
        assert_eq!(lazy.lifecycle, LifecycleState::Registered);
        assert!(lazy.instance.is_none());
    }

    handle
        .resolve_registered_manager::<TestManager>(&current_lazy_identity)
        .unwrap();
    assert_eq!(lazy_calls.load(Ordering::SeqCst), 1);
    assert!(matches!(
        handle.resolve_registered_manager::<TestManager>(&first_immediate_identity),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    assert!(matches!(
        handle.resolve_registered_manager::<TestManager>(&first_lazy_identity),
        Err(CoreError::StaleServiceHandle { .. })
    ));
}

fn activate(runtime: &CoreRuntime, module_name: &str, batch: bool) {
    if batch {
        runtime.activate_registered_modules().unwrap();
    } else {
        runtime.activate_module(module_name).unwrap();
    }
}

fn manager_name(module_name: &str, local_name: &str) -> RegistryName {
    RegistryName::from_parts(module_name, ServiceKind::Manager, local_name)
}

fn counted_manager(
    name: RegistryName,
    startup_mode: StartupMode,
    calls: Arc<AtomicUsize>,
) -> ManagerDescriptor {
    ManagerDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(TestManager) as ServiceObject)
        }),
    )
}

#[derive(Default)]
struct ArmableFinishFailure {
    fail_next: AtomicBool,
}

impl ArmableFinishFailure {
    fn fail_next_finish(&self) {
        self.fail_next.store(true, Ordering::SeqCst);
    }
}

impl ModuleLifecycle for ArmableFinishFailure {
    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(CoreError::MissingConfig("reactivation.finish".to_owned()));
        }
        Ok(())
    }
}
