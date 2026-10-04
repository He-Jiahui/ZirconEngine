use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::super::super::super::super::*;
use crate::core::runtime::{CoreWeak, ServiceObject};
use crate::core::{
    CoreError, CoreResult, LifecycleState, ModuleContext, ModuleLifecycle, ServiceKind, StartupMode,
};

fn register_probe(runtime: &CoreRuntime, module: &str, service: &str) -> String {
    let name = RegistryName::from_parts(module, ServiceKind::Manager, service);
    let full_name = name.as_str().to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new(module, "service-drop probe").with_manager(
                ManagerDescriptor::new(
                    name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    full_name
}

struct ReentrantDrop {
    core: CoreWeak,
    target: String,
    observed: mpsc::Sender<bool>,
}

impl Drop for ReentrantDrop {
    fn drop(&mut self) {
        let core = self.core.clone();
        let target = self.target.clone();
        let (resolved_tx, resolved_rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = resolved_tx.send(core.resolve_manager::<()>(target.as_str()).is_ok());
        });
        let resolved_before_drop_return = resolved_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap_or(false);
        let _ = self.observed.send(resolved_before_drop_return);
    }
}

#[test]
fn service_instance_drop_can_reenter_resolution_after_registry_unlock() {
    let runtime = CoreRuntime::new();
    let target_name = register_probe(&runtime, "RetireReentryProbe", "Probe");
    let (drop_tx, drop_rx) = mpsc::channel();
    let captured_target = target_name.clone();
    let drop_module = "RetireReentryModule";
    let drop_name = RegistryName::from_parts(drop_module, ServiceKind::Manager, "Dropper");
    runtime
        .register_module(
            ModuleDescriptor::new(drop_module, "reentrant service drop").with_manager(
                ManagerDescriptor::new(
                    drop_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |core| {
                        Ok(Arc::new(ReentrantDrop {
                            core: core.clone(),
                            target: captured_target.clone(),
                            observed: drop_tx.clone(),
                        }) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("RetireReentryProbe").unwrap();
    runtime.activate_module(drop_module).unwrap();

    let handle = runtime.handle();
    let deactivation = thread::spawn(move || handle.deactivate_module(drop_module));
    let reentered = drop_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("service Drop must finish its reentrant resolution attempt");
    assert!(deactivation.join().unwrap().is_ok());
    assert!(
        reentered,
        "CoreWeak resolution must finish before the destructor returns"
    );
}

struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("intentional service destructor panic");
    }
}

#[test]
fn panicking_service_instance_drop_does_not_poison_registry_mutex() {
    let runtime = CoreRuntime::new();
    let probe_name = register_probe(&runtime, "RetirePanicProbe", "Probe");
    let module = "RetirePanicModule";
    let name = RegistryName::from_parts(module, ServiceKind::Manager, "Panicker");
    runtime
        .register_module(
            ModuleDescriptor::new(module, "panicking service drop").with_manager(
                ManagerDescriptor::new(
                    name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(PanicOnDrop) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("RetirePanicProbe").unwrap();
    runtime.activate_module(module).unwrap();

    assert!(matches!(
        runtime.deactivate_module(module),
        Err(CoreError::ServiceRetirementPanicked {
            module: failed,
            service,
            command,
        }) if failed == module
            && service == RegistryName::from_parts(module, ServiceKind::Manager, "Panicker").to_string()
            && command == "deactivate"
    ));

    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    drop(services);
    assert!(runtime.resolve_manager::<()>(probe_name.as_str()).is_ok());
}

struct BlockingDrop {
    started: mpsc::Sender<()>,
    release: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl Drop for BlockingDrop {
    fn drop(&mut self) {
        let _ = self.started.send(());
        let _ = self.release.lock().unwrap().recv();
    }
}

#[test]
fn resolve_completes_while_retired_service_drop_is_blocked() {
    let runtime = CoreRuntime::new();
    let probe_name = register_probe(&runtime, "RetireConcurrentProbe", "Probe");
    let drop_module = "RetireConcurrentModule";
    let drop_name = RegistryName::from_parts(drop_module, ServiceKind::Manager, "Blocker");
    let (drop_started_tx, drop_started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let captured_release = Arc::clone(&release_rx);
    runtime
        .register_module(
            ModuleDescriptor::new(drop_module, "blocked service drop").with_manager(
                ManagerDescriptor::new(
                    drop_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(BlockingDrop {
                            started: drop_started_tx.clone(),
                            release: Arc::clone(&captured_release),
                        }) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("RetireConcurrentProbe").unwrap();
    runtime.activate_module(drop_module).unwrap();

    let handle = runtime.handle();
    let deactivation = thread::spawn(move || handle.deactivate_module(drop_module));
    drop_started_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the retired service destructor should start");

    let core = runtime.weak();
    let target = probe_name.clone();
    let (resolver_started_tx, resolver_started_rx) = mpsc::channel();
    let (resolved_tx, resolved_rx) = mpsc::channel();
    let resolver = thread::spawn(move || {
        resolver_started_tx.send(()).unwrap();
        let resolved = core.resolve_manager::<()>(target.as_str()).is_ok();
        resolved_tx.send(resolved).unwrap();
    });
    resolver_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("resolver thread should be scheduled");
    let first_result = resolved_rx.recv_timeout(Duration::from_secs(1));
    let completed_while_drop_blocked = first_result.as_ref().is_ok_and(|resolved| *resolved);

    release_tx.send(()).unwrap();
    let eventual_result = match first_result {
        Ok(resolved) => resolved,
        Err(_) => resolved_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
    };
    resolver.join().unwrap();
    assert!(deactivation.join().unwrap().is_ok());
    assert!(eventual_result);
    assert!(
        completed_while_drop_blocked,
        "service resolution must proceed before the retired destructor is released"
    );
}

#[derive(Debug)]
struct FailingFinish {
    fail: Arc<AtomicBool>,
}

impl ModuleLifecycle for FailingFinish {
    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        if self.fail.load(Ordering::Acquire) {
            Err(CoreError::MissingConfig(
                "retirement finish failure".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

fn register_failing_reentrant_module(
    runtime: &CoreRuntime,
    module: &str,
    target: &str,
    fail: Arc<AtomicBool>,
    observed: mpsc::Sender<bool>,
) -> RegistryName {
    let name = RegistryName::from_parts(module, ServiceKind::Manager, "Dropper");
    let captured_target = target.to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new(module, "failure reset retirement")
                .with_lifecycle(Arc::new(FailingFinish { fail }))
                .with_manager(ManagerDescriptor::new(
                    name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |core| {
                        Ok(Arc::new(ReentrantDrop {
                            core: core.clone(),
                            target: captured_target.clone(),
                            observed: observed.clone(),
                        }) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    name
}

#[test]
fn failed_activation_reset_drops_instances_after_registry_unlock() {
    let runtime = CoreRuntime::new();
    let probe = "RetireFailedActivationProbe";
    let target = register_probe(&runtime, probe, "Probe");
    let module = "RetireFailedActivationModule";
    let (drop_tx, drop_rx) = mpsc::channel();
    let service = register_failing_reentrant_module(
        &runtime,
        module,
        &target,
        Arc::new(AtomicBool::new(true)),
        drop_tx,
    );
    runtime.activate_module(probe).unwrap();
    assert!(runtime.activate_module(module).is_err());
    assert!(drop_rx.recv_timeout(Duration::from_secs(3)).unwrap());
    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service.as_str()).unwrap();
    assert!(entry.instance.is_none());
    assert_eq!(entry.lifecycle, LifecycleState::Registered);
    assert!(!entry.admission_open);
}

#[test]
fn failed_reactivation_reset_drops_instances_after_registry_unlock() {
    let runtime = CoreRuntime::new();
    let probe = "RetireFailedReactivationProbe";
    let target = register_probe(&runtime, probe, "Probe");
    let module = "RetireFailedReactivationModule";
    let fail = Arc::new(AtomicBool::new(false));
    let (drop_tx, drop_rx) = mpsc::channel();
    let service =
        register_failing_reentrant_module(&runtime, module, &target, Arc::clone(&fail), drop_tx);
    runtime.activate_module(probe).unwrap();
    runtime.activate_module(module).unwrap();
    runtime.deactivate_module(module).unwrap();
    assert!(drop_rx.recv_timeout(Duration::from_secs(3)).unwrap());
    let handle = runtime.handle();
    let old_generation = handle
        .inner
        .services
        .lock()
        .unwrap()
        .get(service.as_str())
        .unwrap()
        .generation;
    fail.store(true, Ordering::Release);
    assert!(runtime.activate_module(module).is_err());
    assert!(drop_rx.recv_timeout(Duration::from_secs(3)).unwrap());
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service.as_str()).unwrap();
    assert!(entry.instance.is_none());
    assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
    assert_ne!(entry.generation, old_generation);
    assert!(!entry.admission_open);
}

#[test]
fn failed_batch_reset_drops_instances_after_registry_unlock() {
    let runtime = CoreRuntime::new();
    let probe = "RetireFailedBatchProbe";
    let target = register_probe(&runtime, probe, "Probe");
    let module = "RetireFailedBatchModule";
    let (drop_tx, drop_rx) = mpsc::channel();
    let service = register_failing_reentrant_module(
        &runtime,
        module,
        &target,
        Arc::new(AtomicBool::new(true)),
        drop_tx,
    );
    runtime.activate_module(probe).unwrap();
    assert!(runtime.activate_registered_modules().is_err());
    assert!(drop_rx.recv_timeout(Duration::from_secs(3)).unwrap());
    let handle = runtime.handle();
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service.as_str()).unwrap();
    assert!(entry.instance.is_none());
    assert_eq!(entry.lifecycle, LifecycleState::Registered);
    assert!(!entry.admission_open);
}

#[test]
fn typed_retirement_panic_is_reported_and_census_remains_retryable() {
    let runtime = CoreRuntime::new();
    let _probe_name = register_probe(&runtime, "RetirePanicRetryProbe", "Probe");
    let module = "RetirePanicRetryModule";
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "Panicker");
    let service_key = service.as_str().to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new(module, "retryable panicking service drop").with_manager(
                ManagerDescriptor::new(
                    service,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(PanicOnDrop) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("RetirePanicRetryProbe").unwrap();
    runtime.activate_module(module).unwrap();

    let error = runtime
        .deactivate_module(module)
        .expect_err("the first retirement must report the user destructor panic");
    assert!(matches!(
        error,
        CoreError::ServiceRetirementPanicked {
            module: failed_module,
            service: failed_service,
            command: "deactivate",
        } if failed_module == module && failed_service == service_key
    ));

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules
                .get(module)
                .expect("module remains registered")
                .lifecycle,
            LifecycleState::Stopping
        );
        assert_eq!(
            handle.active_module_shutdown_order(),
            vec![module.to_owned()]
        );
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(service_key.as_str())
            .expect("retired service remains registered");
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime
        .deactivate_module(module)
        .expect("retry must finish teardown after the panic was contained");
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get(module).unwrap().lifecycle,
        LifecycleState::Unloaded
    );
    drop(modules);
    assert!(handle.active_module_shutdown_order().is_empty());
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service_key.as_str()).unwrap();
    assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
    assert!(entry.instance.is_none());
    assert!(!entry.admission_open);
}
