use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::runtime::{ServiceCallGuard, ServiceHandle, ServiceObject};
use crate::core::{
    CoreError, CoreHandle, CoreResult, CoreRuntime, CoreWeak, DriverDescriptor, LifecycleState,
    ModuleCleanupContext, ModuleContext, ModuleDescriptor, ModuleLifecycle, RegistryName,
    ServiceKind, StartupMode,
};

const TEST_WAIT: Duration = Duration::from_secs(5);

#[derive(Debug)]
struct OwnedDriver {
    serial: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct ServiceSnapshot {
    index: u32,
    generation: u32,
    lifecycle: LifecycleState,
    admission_open: bool,
    in_flight_calls: usize,
    initialization_owned: bool,
    has_instance: bool,
}

fn service_snapshot(core: &CoreHandle, name: &str) -> ServiceSnapshot {
    let services = core.inner.services.lock().unwrap();
    let entry = services
        .get(name)
        .expect("the real registered service slot");
    ServiceSnapshot {
        index: entry.index,
        generation: entry.generation,
        lifecycle: entry.lifecycle,
        admission_open: entry.admission_open,
        in_flight_calls: entry.in_flight_calls,
        initialization_owned: entry.initialization_owner.is_some(),
        has_instance: entry.instance.is_some(),
    }
}

fn module_state(core: &CoreHandle, name: &str) -> LifecycleState {
    core.inner
        .modules
        .lock()
        .unwrap()
        .get(name)
        .unwrap()
        .lifecycle
}

struct OwnedLifecycle {
    service_name: String,
    fail_build_call: usize,
    panic_build: bool,
    hold_failed_call: bool,
    build_calls: AtomicUsize,
    cleanup_calls: AtomicUsize,
    fail_cleanup_once: AtomicBool,
    core: Mutex<Option<CoreWeak>>,
    saved_handle: Mutex<Option<ServiceHandle<OwnedDriver>>>,
    held_call: Mutex<Option<ServiceCallGuard<OwnedDriver>>>,
    expected_pointer: AtomicUsize,
    cleanup_serials: Mutex<Vec<usize>>,
    absent_name: Option<String>,
    foreign_name: Option<String>,
}

impl OwnedLifecycle {
    fn new(service_name: String) -> Self {
        Self {
            service_name,
            fail_build_call: 0,
            panic_build: false,
            hold_failed_call: false,
            build_calls: AtomicUsize::new(0),
            cleanup_calls: AtomicUsize::new(0),
            fail_cleanup_once: AtomicBool::new(false),
            core: Mutex::new(None),
            saved_handle: Mutex::new(None),
            held_call: Mutex::new(None),
            expected_pointer: AtomicUsize::new(0),
            cleanup_serials: Mutex::new(Vec::new()),
            absent_name: None,
            foreign_name: None,
        }
    }
}

impl ModuleLifecycle for OwnedLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        let call = self.build_calls.fetch_add(1, Ordering::SeqCst) + 1;
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        *self.core.lock().unwrap() = Some(core.downgrade());
        let handle = core.resolve_driver_handle::<OwnedDriver>(&self.service_name)?;
        let guard = handle.enter()?;
        self.expected_pointer
            .store((&*guard as *const OwnedDriver) as usize, Ordering::SeqCst);
        *self.saved_handle.lock().unwrap() = Some(handle);
        if call == self.fail_build_call {
            if self.hold_failed_call {
                *self.held_call.lock().unwrap() = Some(guard);
            } else {
                drop(guard);
            }
            if self.panic_build {
                panic!("controlled real registry build panic");
            }
            return Err(CoreError::MissingConfig("owner.build".to_owned()));
        }
        drop(guard);
        Ok(())
    }

    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
        let core = self
            .core
            .lock()
            .unwrap()
            .as_ref()
            .and_then(CoreWeak::upgrade)
            .ok_or(CoreError::RuntimeUnavailable)?;
        assert!(
            core.inner.modules.try_lock().is_ok(),
            "module callback outside module lock"
        );
        assert!(
            core.inner.services.try_lock().is_ok(),
            "module callback outside registry lock"
        );
        assert!(matches!(
            core.resolve_driver::<OwnedDriver>(&self.service_name),
            Err(CoreError::ServiceUnavailable(_))
        ));
        assert!(matches!(
            self.saved_handle.lock().unwrap().as_ref().unwrap().enter(),
            Err(CoreError::ServiceUnavailable(_))
        ));
        assert!(
            context.with_existing_driver::<OwnedDriver>(&self.service_name, |driver| {
                assert_eq!(
                    driver as *const OwnedDriver as usize,
                    self.expected_pointer.load(Ordering::SeqCst)
                );
                let row = service_snapshot(&core, &self.service_name);
                assert!(!row.admission_open);
                assert_eq!(row.in_flight_calls, 0);
                assert!(!row.initialization_owned);
                self.cleanup_serials.lock().unwrap().push(driver.serial);
                Ok(())
            })?
        );
        if let Some(name) = &self.absent_name {
            assert!(!context.with_existing_driver::<OwnedDriver>(name, |_| {
                panic!("an absent lazy service cannot be initialized by cleanup");
            })?);
        }
        if let Some(name) = &self.foreign_name {
            assert!(matches!(
                context.with_existing_driver::<OwnedDriver>(name, |_| {
                    panic!("cleanup cannot borrow another module's service");
                }),
                Err(CoreError::ServiceUnavailable(_))
            ));
        }
        assert!(matches!(
            context.with_existing_manager::<OwnedDriver>(&self.service_name, |_| Ok(())),
            Err(CoreError::ServiceKindMismatch { .. })
        ));
        assert!(matches!(
            context.with_existing_driver::<u64>(&self.service_name, |_| Ok(())),
            Err(CoreError::ServiceDowncast(_))
        ));
        if self.fail_cleanup_once.swap(false, Ordering::SeqCst) {
            return Err(CoreError::MissingConfig("owner.cleanup".to_owned()));
        }
        Ok(())
    }
}

fn owner_runtime(
    module: &str,
    lifecycle: Arc<OwnedLifecycle>,
    factory_calls: Arc<AtomicUsize>,
) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(module, ServiceKind::Driver, "Owned");
    let mut descriptor = ModuleDescriptor::new(module, "real registry cleanup owner")
        .with_lifecycle(lifecycle.clone())
        .with_driver(DriverDescriptor::new(
            service_name,
            StartupMode::Lazy,
            Vec::new(),
            Arc::new(move |_| {
                let serial = factory_calls.fetch_add(1, Ordering::SeqCst) + 1;
                Ok(Arc::new(OwnedDriver { serial }) as ServiceObject)
            }),
        ));
    if let Some(name) = &lifecycle.absent_name {
        descriptor = descriptor.with_driver(DriverDescriptor::new(
            RegistryName::new(name.as_str()).expect("owned lazy name"),
            StartupMode::Lazy,
            Vec::new(),
            Arc::new(|_| panic!("cleanup must not invoke an absent lazy factory")),
        ));
    }
    runtime.register_module(descriptor).unwrap();
    if let Some(name) = &lifecycle.foreign_name {
        runtime
            .register_module(
                ModuleDescriptor::new("ForeignOwner", "foreign capability boundary").with_driver(
                    DriverDescriptor::new(
                        RegistryName::new(name.as_str()).expect("foreign service name"),
                        StartupMode::Lazy,
                        Vec::new(),
                        Arc::new(|_| Ok(Arc::new(OwnedDriver { serial: 99 }) as ServiceObject)),
                    ),
                ),
            )
            .unwrap();
    }
    runtime
}

fn saved_handle(lifecycle: &OwnedLifecycle) -> ServiceHandle<OwnedDriver> {
    lifecycle
        .saved_handle
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone()
}

fn assert_drain_rollback(error: CoreError, panic_build: bool, batch: bool) {
    let (activation, cleanup) = if batch {
        match error {
            CoreError::ModuleBatchActivationRollback {
                activation,
                cleanup_failures,
            } => {
                assert_eq!(cleanup_failures.len(), 1);
                (
                    activation,
                    Box::new(cleanup_failures.into_iter().next().unwrap().1),
                )
            }
            other => panic!("expected batch drain rollback, found {other:?}"),
        }
    } else {
        match error {
            CoreError::ModuleActivationRollback {
                activation,
                cleanup,
            } => (activation, cleanup),
            other => panic!("expected single drain rollback, found {other:?}"),
        }
    };
    if panic_build {
        assert!(matches!(
            *activation,
            CoreError::ModuleLifecycleCallbackPanicked { .. }
        ));
    } else {
        assert!(matches!(*activation, CoreError::MissingConfig(ref key) if key == "owner.build"));
    }
    assert!(matches!(
        *cleanup,
        CoreError::ServiceCallDrainTimeout {
            in_flight_calls: 1,
            ..
        }
    ));
}

#[test]
fn cleanup_owner_retry_borrows_same_registry_instance_with_external_admission_closed() {
    let module = "CleanupOwnerRetry";
    let name = format!("{module}.Driver.Owned");
    let mut fixture = OwnedLifecycle::new(name.clone());
    fixture.fail_cleanup_once = AtomicBool::new(true);
    fixture.absent_name = Some(format!("{module}.Driver.Absent"));
    fixture.foreign_name = Some("ForeignOwner.Driver.Foreign".to_owned());
    let lifecycle = Arc::new(fixture);
    let factories = Arc::new(AtomicUsize::new(0));
    let runtime = owner_runtime(module, lifecycle.clone(), factories.clone());
    runtime.activate_module(module).unwrap();
    let core = runtime.handle();
    let before = service_snapshot(&core, &name);
    let old = saved_handle(&lifecycle);

    assert!(matches!(
        runtime.deactivate_module(module),
        Err(CoreError::MissingConfig(ref key)) if key == "owner.cleanup"
    ));
    let retained = service_snapshot(&core, &name);
    assert_eq!(module_state(&core, module), LifecycleState::Stopping);
    assert_eq!(retained.generation, before.generation);
    assert_eq!(retained.index, before.index);
    assert_eq!(retained.lifecycle, LifecycleState::Stopping);
    assert!(!retained.admission_open);
    assert!(retained.has_instance);
    assert_eq!(retained.in_flight_calls, 0);
    assert!(!retained.initialization_owned);
    assert!(
        !core
            .inner
            .modules
            .lock()
            .unwrap()
            .get(module)
            .unwrap()
            .cleanup_completed
    );
    assert_eq!(factories.load(Ordering::SeqCst), 1);

    let retained_context = ModuleCleanupContext::new(&core, module).unwrap();
    // Only the retry enters product cleanup again; ordinary admission stays closed.
    runtime.deactivate_module(module).unwrap();
    assert_eq!(lifecycle.cleanup_serials.lock().unwrap().as_slice(), [1, 1]);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    assert!(matches!(
        old.enter(),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    assert_eq!(module_state(&core, module), LifecycleState::Unloaded);
    assert!(matches!(
        retained_context.with_existing_driver::<OwnedDriver>(&name, |_| {
            panic!("an old cleanup generation cannot borrow a retired instance");
        }),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    runtime.activate_module(module).unwrap();
    assert_eq!(factories.load(Ordering::SeqCst), 2);
    assert!(matches!(
        retained_context.with_existing_driver::<OwnedDriver>(&name, |_| {
            panic!("an old cleanup generation cannot borrow a replacement instance");
        }),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    assert_eq!(factories.load(Ordering::SeqCst), 2);
    runtime.deactivate_module(module).unwrap();
}

fn build_failure_with_live_call(panic_build: bool, batch: bool, reactivation: bool) {
    let module = "PartialBuildOwner";
    let name = format!("{module}.Driver.Owned");
    let mut fixture = OwnedLifecycle::new(name.clone());
    fixture.fail_build_call = if reactivation { 2 } else { 1 };
    fixture.panic_build = panic_build;
    fixture.hold_failed_call = true;
    let lifecycle = Arc::new(fixture);
    let factories = Arc::new(AtomicUsize::new(0));
    let runtime = owner_runtime(module, lifecycle.clone(), factories.clone());
    let core = runtime.handle();
    if reactivation {
        runtime.activate_module(module).unwrap();
        runtime.deactivate_module(module).unwrap();
    }
    let original_generation = service_snapshot(&core, &name).generation;
    let previous_cleanups = lifecycle.cleanup_calls.load(Ordering::SeqCst);
    let error = if batch {
        runtime.activate_registered_modules().unwrap_err()
    } else {
        runtime.activate_module(module).unwrap_err()
    };
    assert_drain_rollback(error, panic_build, batch);

    let retained = service_snapshot(&core, &name);
    assert_eq!(module_state(&core, module), LifecycleState::Stopping);
    assert_eq!(retained.lifecycle, LifecycleState::Stopping);
    assert_eq!(retained.generation, original_generation);
    assert!(!retained.admission_open);
    assert_eq!(retained.in_flight_calls, 1);
    assert!(!retained.initialization_owned);
    assert!(retained.has_instance);
    assert_eq!(
        lifecycle.cleanup_calls.load(Ordering::SeqCst),
        previous_cleanups
    );
    assert_eq!(core.active_module_shutdown_order(), vec![module]);
    let old = saved_handle(&lifecycle);
    assert!(matches!(old.enter(), Err(CoreError::ServiceUnavailable(_))));

    drop(lifecycle.held_call.lock().unwrap().take());
    let drained = service_snapshot(&core, &name);
    assert_eq!(drained.in_flight_calls, 0);
    assert_eq!(drained.generation, original_generation);
    runtime.deactivate_module(module).unwrap();
    assert_eq!(
        lifecycle.cleanup_calls.load(Ordering::SeqCst),
        previous_cleanups + 1
    );
    assert!(matches!(
        old.enter(),
        Err(CoreError::StaleServiceHandle { .. })
    ));

    runtime.activate_module(module).unwrap();
    let current = core.resolve_driver_handle::<OwnedDriver>(&name).unwrap();
    let new_call = current.enter().unwrap();
    let new_generation = service_snapshot(&core, &name).generation;
    assert_ne!(new_generation, original_generation);
    assert!(matches!(
        old.enter(),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    drop(old);
    let live = service_snapshot(&core, &name);
    assert_eq!(live.generation, new_generation);
    assert_eq!(
        live.in_flight_calls, 1,
        "stale-generation handle cannot alter the new call"
    );
    drop(new_call);
    assert_eq!(service_snapshot(&core, &name).in_flight_calls, 0);
    runtime.deactivate_module(module).unwrap();
}

#[test]
fn failed_build_error_retains_live_call_generation_until_real_drain_and_cleanup() {
    build_failure_with_live_call(false, false, false);
}

#[test]
fn failed_build_panic_retains_live_call_generation_until_real_drain_and_cleanup() {
    build_failure_with_live_call(true, false, false);
}

#[test]
fn failed_batch_build_retains_live_call_generation_until_real_drain_and_cleanup() {
    build_failure_with_live_call(false, true, false);
}

#[test]
fn failed_reactivation_build_retains_live_call_generation_until_real_drain_and_cleanup() {
    build_failure_with_live_call(false, false, true);
}

#[test]
fn failed_build_without_live_calls_cleans_actual_lazy_instance_before_generation_reset() {
    for panic_build in [false, true] {
        let module = "BuildCleanupBeforeReset";
        let name = format!("{module}.Driver.Owned");
        let mut fixture = OwnedLifecycle::new(name.clone());
        fixture.fail_build_call = 1;
        fixture.panic_build = panic_build;
        let lifecycle = Arc::new(fixture);
        let factories = Arc::new(AtomicUsize::new(0));
        let runtime = owner_runtime(module, lifecycle.clone(), factories.clone());
        let core = runtime.handle();
        let generation = service_snapshot(&core, &name).generation;
        let error = runtime.activate_module(module).unwrap_err();
        if panic_build {
            assert!(matches!(
                error,
                CoreError::ModuleLifecycleCallbackPanicked { .. }
            ));
        } else {
            assert!(matches!(error, CoreError::MissingConfig(ref key) if key == "owner.build"));
        }
        assert_eq!(lifecycle.cleanup_serials.lock().unwrap().as_slice(), [1]);
        let reset = service_snapshot(&core, &name);
        assert_eq!(module_state(&core, module), LifecycleState::Registered);
        assert_eq!(reset.lifecycle, LifecycleState::Registered);
        assert!(!reset.has_instance);
        assert_eq!(reset.in_flight_calls, 0);
        assert!(!reset.initialization_owned);
        assert_ne!(reset.generation, generation);
        assert!(core.active_module_shutdown_order().is_empty());
        runtime.activate_module(module).unwrap();
        assert_eq!(factories.load(Ordering::SeqCst), 2);
        runtime.deactivate_module(module).unwrap();
    }
}

struct ProjectedDropWitness {
    core: CoreWeak,
    name: String,
    observed_calls: Arc<AtomicUsize>,
}

impl Drop for ProjectedDropWitness {
    fn drop(&mut self) {
        let core = self.core.upgrade().expect("the guard retains its runtime");
        self.observed_calls.store(
            service_snapshot(&core, &self.name).in_flight_calls,
            Ordering::SeqCst,
        );
    }
}

#[test]
fn mapped_guard_drops_projected_object_before_releasing_real_call_admission() {
    let runtime = CoreRuntime::new();
    let module = "MappedGuardOwner";
    let name = format!("{module}.Driver.Owned");
    runtime
        .register_module(
            ModuleDescriptor::new(module, "projected guard drop order").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts(module, ServiceKind::Driver, "Owned"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(OwnedDriver { serial: 1 }) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module(module).unwrap();
    let core = runtime.handle();
    let observed = Arc::new(AtomicUsize::new(usize::MAX));
    let guard = core.resolve_driver::<OwnedDriver>(&name).unwrap();
    let mapped = guard.map(|_| {
        Arc::new(ProjectedDropWitness {
            core: core.downgrade(),
            name: name.clone(),
            observed_calls: observed.clone(),
        })
    });
    assert_eq!(service_snapshot(&core, &name).in_flight_calls, 1);
    drop(mapped);
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert_eq!(service_snapshot(&core, &name).in_flight_calls, 0);
    runtime.deactivate_module(module).unwrap();
}

struct DelayedDriver {
    drop_started: mpsc::SyncSender<()>,
    release_drop: Mutex<mpsc::Receiver<()>>,
    drop_finished: mpsc::SyncSender<bool>,
}

impl Drop for DelayedDriver {
    fn drop(&mut self) {
        let _ = self.drop_started.send(());
        let explicitly_released = self
            .release_drop
            .lock()
            .unwrap()
            .recv_timeout(TEST_WAIT)
            .is_ok();
        let _ = self.drop_finished.send(explicitly_released);
    }
}

struct AbsentCleanupLifecycle {
    service_name: String,
    cleanup_calls: Arc<AtomicUsize>,
}

impl ModuleLifecycle for AbsentCleanupLifecycle {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !context.with_existing_driver::<DelayedDriver>(&self.service_name, |_| {
                panic!("a late rejected object cannot become the registered cleanup instance");
            })?
        );
        Ok(())
    }
}

#[test]
fn late_factory_object_drop_settles_before_initialization_claim_allows_cleanup() {
    let runtime = CoreRuntime::new();
    let module = "LateFactoryOwner";
    let name = format!("{module}.Driver.Delayed");
    let (factory_started_tx, factory_started_rx) = mpsc::sync_channel(1);
    let (release_factory_tx, release_factory_rx) = mpsc::sync_channel(1);
    let (drop_started_tx, drop_started_rx) = mpsc::sync_channel(1);
    let (release_drop_tx, release_drop_rx) = mpsc::sync_channel(1);
    let (drop_finished_tx, drop_finished_rx) = mpsc::sync_channel(1);
    let release_factory = Mutex::new(release_factory_rx);
    let release_drop = Mutex::new(Some(release_drop_rx));
    let factories = Arc::new(AtomicUsize::new(0));
    let factory_count = factories.clone();
    let cleanups = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new(module, "late factory release barrier")
                .with_lifecycle(Arc::new(AbsentCleanupLifecycle {
                    service_name: name.clone(),
                    cleanup_calls: cleanups.clone(),
                }))
                .with_driver(DriverDescriptor::new(
                    RegistryName::from_parts(module, ServiceKind::Driver, "Delayed"),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        factory_count.fetch_add(1, Ordering::SeqCst);
                        let driver = Arc::new(DelayedDriver {
                            drop_started: drop_started_tx.clone(),
                            release_drop: Mutex::new(release_drop.lock().unwrap().take().unwrap()),
                            drop_finished: drop_finished_tx.clone(),
                        });
                        factory_started_tx.send(()).unwrap();
                        release_factory
                            .lock()
                            .unwrap()
                            .recv_timeout(TEST_WAIT)
                            .map_err(|_| {
                                CoreError::MissingConfig("test.factory.deadline".to_owned())
                            })?;
                        Ok(driver as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime.activate_module(module).unwrap();
    let core = runtime.handle();
    let initial = service_snapshot(&core, &name);
    let (resolved_tx, resolved_rx) = mpsc::sync_channel(1);
    let resolver_core = core.clone();
    let resolver_name = name.clone();
    let resolver = thread::spawn(move || {
        let result = resolver_core.resolve_driver_handle::<DelayedDriver>(&resolver_name);
        let _ = resolved_tx.send(result);
    });
    factory_started_rx
        .recv_timeout(TEST_WAIT)
        .expect("actual factory constructed its object");
    assert!(matches!(
        runtime.deactivate_module_with_drain_timeout(module, Duration::ZERO),
        Err(CoreError::ServiceCallDrainTimeout {
            in_flight_calls: 1,
            ..
        })
    ));
    let fenced = service_snapshot(&core, &name);
    assert_eq!(fenced.lifecycle, LifecycleState::Stopping);
    assert_eq!(fenced.generation, initial.generation);
    assert_eq!(fenced.index, initial.index);
    assert!(!fenced.admission_open);
    assert!(fenced.initialization_owned);
    assert!(!fenced.has_instance);
    assert_eq!(cleanups.load(Ordering::SeqCst), 0);

    release_factory_tx.send(()).unwrap();
    drop_started_rx
        .recv_timeout(TEST_WAIT)
        .expect("actual rejected object Drop began");
    assert!(matches!(
        runtime.deactivate_module_with_drain_timeout(module, Duration::ZERO),
        Err(CoreError::ServiceCallDrainTimeout {
            in_flight_calls: 1,
            ..
        })
    ));
    assert!(service_snapshot(&core, &name).initialization_owned);
    assert_eq!(cleanups.load(Ordering::SeqCst), 0);
    assert!(matches!(
        resolved_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));

    release_drop_tx.send(()).unwrap();
    assert!(drop_finished_rx
        .recv_timeout(TEST_WAIT)
        .expect("explicit actual Drop release"));
    let resolution = resolved_rx
        .recv_timeout(TEST_WAIT)
        .expect("resolver terminal result");
    assert!(matches!(resolution, Err(CoreError::ServiceUnavailable(_))));
    resolver
        .join()
        .expect("the bounded completed resolver cannot panic");
    let settled = service_snapshot(&core, &name);
    assert_eq!(settled.generation, initial.generation);
    assert_eq!(settled.lifecycle, LifecycleState::Stopping);
    assert!(!settled.initialization_owned);
    assert!(!settled.has_instance);
    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanups.load(Ordering::SeqCst), 1);
    assert_eq!(factories.load(Ordering::SeqCst), 1);
    assert_eq!(module_state(&core, module), LifecycleState::Unloaded);
}

#[test]
#[cfg(feature = "text")]
fn registered_text_cleanup_keeps_draining_family_and_same_generation_for_retry() {
    use crate::core::manager::RegisteredManagerService;
    use crate::engine_module::EngineModule;
    use crate::text::{
        text_runtime_context_for_core, TextModule, TextRuntimeContext,
        TextRuntimeContextLifecycleState, TEXT_MODULE_NAME,
    };
    const TEXT_RUNTIME_CONTEXT_MANAGER_NAME: &str = "TextModule.Manager.FontServices";

    let runtime = CoreRuntime::new();
    runtime
        .register_module(TextModule::default().descriptor())
        .unwrap();
    runtime.activate_module(TEXT_MODULE_NAME).unwrap();
    let core = runtime.handle();
    let guard = text_runtime_context_for_core(&core).unwrap();
    let family = guard.create_layout_session().unwrap();
    drop(guard);
    let before = service_snapshot(&core, TEXT_RUNTIME_CONTEXT_MANAGER_NAME);
    assert_eq!(before.in_flight_calls, 0);

    assert!(matches!(
        runtime.deactivate_module(TEXT_MODULE_NAME),
        Err(CoreError::ModuleCleanupTimeout {
            incomplete_entries: 1,
            failed: 0,
            ..
        })
    ));
    let retained = service_snapshot(&core, TEXT_RUNTIME_CONTEXT_MANAGER_NAME);
    assert_eq!(
        module_state(&core, TEXT_MODULE_NAME),
        LifecycleState::Stopping
    );
    assert_eq!(retained.generation, before.generation);
    assert!(!retained.admission_open);
    assert!(retained.has_instance);
    assert!(matches!(
        text_runtime_context_for_core(&core),
        Err(CoreError::ServiceUnavailable(_))
    ));
    {
        // This read observes the actual registered instance, with no extra Arc owner.
        let services = core.inner.services.lock().unwrap();
        let registered = services
            .get(TEXT_RUNTIME_CONTEXT_MANAGER_NAME)
            .unwrap()
            .instance
            .as_ref()
            .unwrap()
            .downcast_ref::<RegisteredManagerService<TextRuntimeContext>>()
            .unwrap();
        assert_eq!(
            registered.borrowed().lifecycle_state(),
            TextRuntimeContextLifecycleState::Draining
        );
        assert_eq!(
            registered
                .borrowed()
                .health_snapshot()
                .active_layout_session_family_count(),
            1
        );
    }

    drop(family);
    {
        let services = core.inner.services.lock().unwrap();
        let registered = services
            .get(TEXT_RUNTIME_CONTEXT_MANAGER_NAME)
            .unwrap()
            .instance
            .as_ref()
            .unwrap()
            .downcast_ref::<RegisteredManagerService<TextRuntimeContext>>()
            .unwrap();
        assert_eq!(
            registered.borrowed().lifecycle_state(),
            TextRuntimeContextLifecycleState::Closed
        );
    }
    runtime.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        module_state(&core, TEXT_MODULE_NAME),
        LifecycleState::Unloaded
    );
    assert!(!service_snapshot(&core, TEXT_RUNTIME_CONTEXT_MANAGER_NAME).has_instance);
}
