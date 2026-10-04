use super::*;
use crate::core::runtime::{CoreWeak, ServiceObject};
use crate::core::{
    RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver, ServiceKind, StartupMode,
};
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
struct ProviderCleanupSignal {
    cleanup_started: Mutex<Option<mpsc::SyncSender<()>>>,
}

impl ModuleLifecycle for ProviderCleanupSignal {
    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        if let Some(sender) = self.cleanup_started.lock().unwrap().take() {
            sender.send(()).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug)]
struct BlockingDependentBuild {
    build_calls: Arc<AtomicUsize>,
    build_started: Mutex<Option<mpsc::SyncSender<()>>>,
    release_build: Mutex<mpsc::Receiver<()>>,
    provider_visibility: Option<CoreWeak>,
    visibility_observed: Option<Arc<AtomicBool>>,
}

impl ModuleLifecycle for BlockingDependentBuild {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.build_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(core) = self
            .provider_visibility
            .as_ref()
            .and_then(|weak| weak.upgrade())
        {
            let modules = core.inner.modules.lock().unwrap();
            assert_eq!(
                modules
                    .get("ClosureLeaseProvider")
                    .expect("provider must be visible before dependent build")
                    .lifecycle,
                LifecycleState::Running
            );
            drop(modules);
            assert!(core
                .active_module_shutdown_order()
                .iter()
                .any(|module| module == "ClosureLeaseProvider"));
            if let Some(observed) = &self.visibility_observed {
                observed.store(true, Ordering::Release);
            }
        }
        if let Some(sender) = self.build_started.lock().unwrap().take() {
            sender.send(()).unwrap();
        }
        self.release_build.lock().unwrap().recv().unwrap();
        Ok(())
    }
}

#[test]
fn single_activation_holds_provider_until_dependent_is_published_running() {
    let runtime = CoreRuntime::new();
    let (cleanup_started_tx, cleanup_started_rx) = mpsc::sync_channel(1);
    runtime
        .register_module(
            ModuleDescriptor::new("ClosureLeaseProvider", "activation closure reservation")
                .with_lifecycle(Arc::new(ProviderCleanupSignal {
                    cleanup_started: Mutex::new(Some(cleanup_started_tx)),
                })),
        )
        .unwrap();

    let (build_started_tx, build_started_rx) = mpsc::sync_channel(1);
    let (release_build_tx, release_build_rx) = mpsc::sync_channel(1);
    let build_calls = Arc::new(AtomicUsize::new(0));
    let provider_visibility_observed = Arc::new(AtomicBool::new(false));
    runtime
        .register_module(
            ModuleDescriptor::new("ClosureLeaseConsumer", "activation closure reservation")
                .with_module_dependency(ModuleDependencySpec::named("ClosureLeaseProvider"))
                .with_lifecycle(Arc::new(BlockingDependentBuild {
                    build_calls: Arc::clone(&build_calls),
                    build_started: Mutex::new(Some(build_started_tx)),
                    release_build: Mutex::new(release_build_rx),
                    provider_visibility: Some(runtime.weak()),
                    visibility_observed: Some(Arc::clone(&provider_visibility_observed)),
                })),
        )
        .unwrap();

    let activation_runtime = runtime.clone();
    let activation =
        thread::spawn(move || activation_runtime.activate_module("ClosureLeaseConsumer"));
    build_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("dependent callback must start after its provider activation");

    let deactivation_runtime = runtime.clone();
    let (deactivation_started_tx, deactivation_started_rx) = mpsc::sync_channel(1);
    let (deactivation_result_tx, deactivation_result_rx) = mpsc::channel();
    let deactivation = thread::spawn(move || {
        deactivation_started_tx.send(()).unwrap();
        deactivation_result_tx
            .send(deactivation_runtime.deactivate_module("ClosureLeaseProvider"))
            .unwrap();
    });
    deactivation_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("provider deactivation thread must start");

    let handle = runtime.handle();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut activation_owns_provider = false;
    let mut provider_cleanup_started = false;
    while Instant::now() < deadline {
        let opposite_waiters = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .opposite_waiter_count_for_test(
                "ClosureLeaseProvider",
                ModuleLifecycleCommand::Activate,
            );
        if opposite_waiters > 0 {
            activation_owns_provider = true;
            break;
        }
        if cleanup_started_rx.try_recv().is_ok() {
            provider_cleanup_started = true;
            break;
        }
        thread::yield_now();
    }

    release_build_tx
        .send(())
        .expect("release the dependent callback before asserting the interleaving");
    activation
        .join()
        .expect("activation thread must not panic")
        .expect("the complete dependency closure should activate successfully");
    let deactivation_result = deactivation_result_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("provider deactivation should resume after closure completion");
    deactivation
        .join()
        .expect("deactivation thread must not panic");

    assert!(
        activation_owns_provider,
        "the closure must retain the provider transition until dependent publication"
    );
    assert!(
        !provider_cleanup_started,
        "provider cleanup must not begin while its dependent callback is in flight"
    );
    assert!(matches!(
        deactivation_result,
        Err(CoreError::ModuleUnloadBlocked { module, dependents })
            if module == "ClosureLeaseProvider"
                && dependents == vec!["ClosureLeaseConsumer".to_owned()]
    ));
    assert_eq!(build_calls.load(Ordering::SeqCst), 1);
    assert!(provider_visibility_observed.load(Ordering::Acquire));

    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("ClosureLeaseProvider")
            .expect("provider remains registered")
            .lifecycle,
        LifecycleState::Running
    );
    assert_eq!(
        modules
            .get("ClosureLeaseConsumer")
            .expect("consumer remains registered")
            .lifecycle,
        LifecycleState::Running
    );
}

#[derive(Debug)]
struct BlockingProviderLifecycle {
    build_calls: Arc<AtomicUsize>,
    build_started: Mutex<Option<mpsc::SyncSender<()>>>,
    release_build: Mutex<mpsc::Receiver<()>>,
    cleanup_started: Mutex<Option<mpsc::SyncSender<()>>>,
}

impl ModuleLifecycle for BlockingProviderLifecycle {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.build_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(sender) = self.build_started.lock().unwrap().take() {
            sender.send(()).unwrap();
        }
        self.release_build.lock().unwrap().recv().unwrap();
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        if let Some(sender) = self.cleanup_started.lock().unwrap().take() {
            sender.send(()).unwrap();
        }
        Ok(())
    }
}

#[test]
fn joined_provider_activation_reacquires_lease_before_dependent_callback() {
    let runtime = CoreRuntime::new();
    let (cleanup_started_tx, cleanup_started_rx) = mpsc::sync_channel(1);
    let (provider_build_started_tx, provider_build_started_rx) = mpsc::sync_channel(1);
    let (release_provider_tx, release_provider_rx) = mpsc::sync_channel(1);
    let provider_build_calls = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("JoinedClosureProvider", "joined closure reservation")
                .with_lifecycle(Arc::new(BlockingProviderLifecycle {
                    build_calls: Arc::clone(&provider_build_calls),
                    build_started: Mutex::new(Some(provider_build_started_tx)),
                    release_build: Mutex::new(release_provider_rx),
                    cleanup_started: Mutex::new(Some(cleanup_started_tx)),
                })),
        )
        .unwrap();

    let (consumer_build_started_tx, consumer_build_started_rx) = mpsc::sync_channel(1);
    let (release_consumer_tx, release_consumer_rx) = mpsc::sync_channel(1);
    let consumer_build_calls = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("JoinedClosureConsumer", "joined closure reservation")
                .with_module_dependency(ModuleDependencySpec::named("JoinedClosureProvider"))
                .with_lifecycle(Arc::new(BlockingDependentBuild {
                    build_calls: Arc::clone(&consumer_build_calls),
                    build_started: Mutex::new(Some(consumer_build_started_tx)),
                    release_build: Mutex::new(release_consumer_rx),
                    provider_visibility: None,
                    visibility_observed: None,
                })),
        )
        .unwrap();

    let provider_runtime = runtime.clone();
    let provider_activation =
        thread::spawn(move || provider_runtime.activate_module("JoinedClosureProvider"));
    provider_build_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("prior provider activation must enter its blocked callback");

    let consumer_runtime = runtime.clone();
    let consumer_activation =
        thread::spawn(move || consumer_runtime.activate_module("JoinedClosureConsumer"));
    let handle = runtime.handle();
    let join_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let joined = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .waiter_count("JoinedClosureProvider", ModuleLifecycleCommand::Activate);
        if joined == 1 {
            break;
        }
        assert!(
            Instant::now() < join_deadline,
            "closure must join the existing provider activation"
        );
        thread::yield_now();
    }
    release_provider_tx.send(()).unwrap();
    provider_activation.join().unwrap().unwrap();
    consumer_build_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("dependent callback must begin after the joined provider succeeds");

    let deactivation_runtime = runtime.clone();
    let (deactivation_started_tx, deactivation_started_rx) = mpsc::sync_channel(1);
    let (deactivation_result_tx, deactivation_result_rx) = mpsc::channel();
    let deactivation = thread::spawn(move || {
        deactivation_started_tx.send(()).unwrap();
        deactivation_result_tx
            .send(deactivation_runtime.deactivate_module("JoinedClosureProvider"))
            .unwrap();
    });
    deactivation_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("concurrent provider deactivation must start");
    let reservation_deadline = Instant::now() + Duration::from_secs(2);
    let mut activation_reacquired_provider = false;
    let mut provider_cleanup_started = false;
    while Instant::now() < reservation_deadline {
        let opposite_waiters = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .opposite_waiter_count_for_test(
                "JoinedClosureProvider",
                ModuleLifecycleCommand::Activate,
            );
        if opposite_waiters > 0 {
            activation_reacquired_provider = true;
            break;
        }
        if cleanup_started_rx.try_recv().is_ok() {
            provider_cleanup_started = true;
            break;
        }
        thread::yield_now();
    }

    release_consumer_tx.send(()).unwrap();
    consumer_activation.join().unwrap().unwrap();
    let deactivation_result = deactivation_result_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("provider deactivation must finish after closure publication");
    deactivation.join().unwrap();

    assert!(
        activation_reacquired_provider,
        "joined Ok must be followed by a fresh closure reservation"
    );
    assert!(
        !provider_cleanup_started,
        "provider cleanup must not run during dependent initialization"
    );
    assert!(matches!(
        deactivation_result,
        Err(CoreError::ModuleUnloadBlocked { module, dependents })
            if module == "JoinedClosureProvider" && dependents == vec!["JoinedClosureConsumer".to_owned()]
    ));
    assert_eq!(provider_build_calls.load(Ordering::SeqCst), 1);
    assert_eq!(consumer_build_calls.load(Ordering::SeqCst), 1);
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get("JoinedClosureProvider").unwrap().lifecycle,
        LifecycleState::Running
    );
    assert_eq!(
        modules.get("JoinedClosureConsumer").unwrap().lifecycle,
        LifecycleState::Running
    );
}

#[derive(Debug)]
struct BlockingBatchFailure {
    build_started: Mutex<Option<mpsc::SyncSender<()>>>,
    release_build: Mutex<mpsc::Receiver<()>>,
}

impl ModuleLifecycle for BlockingBatchFailure {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        if let Some(sender) = self.build_started.lock().unwrap().take() {
            sender.send(()).unwrap();
        }
        self.release_build.lock().unwrap().recv().unwrap();
        Err(CoreError::MissingConfig("batch.receipt.failure".to_owned()))
    }
}

#[test]
fn batch_failure_preserves_joined_running_provider_success_receipt() {
    let runtime = CoreRuntime::new();
    let (provider_cleanup_tx, _provider_cleanup_rx) = mpsc::sync_channel(1);
    let (provider_started_tx, provider_started_rx) = mpsc::sync_channel(1);
    let (release_provider_tx, release_provider_rx) = mpsc::sync_channel(1);
    let provider_build_calls = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("BatchReceiptProvider", "batch receipt").with_lifecycle(
                Arc::new(BlockingProviderLifecycle {
                    build_calls: Arc::clone(&provider_build_calls),
                    build_started: Mutex::new(Some(provider_started_tx)),
                    release_build: Mutex::new(release_provider_rx),
                    cleanup_started: Mutex::new(Some(provider_cleanup_tx)),
                }),
            ),
        )
        .unwrap();

    let (failure_started_tx, failure_started_rx) = mpsc::sync_channel(1);
    let (release_failure_tx, release_failure_rx) = mpsc::sync_channel(1);
    runtime
        .register_module(
            ModuleDescriptor::new("BatchReceiptFailure", "batch receipt")
                .with_module_dependency(ModuleDependencySpec::named("BatchReceiptProvider"))
                .with_lifecycle(Arc::new(BlockingBatchFailure {
                    build_started: Mutex::new(Some(failure_started_tx)),
                    release_build: Mutex::new(release_failure_rx),
                })),
        )
        .unwrap();

    let provider_runtime = runtime.clone();
    let provider_activation =
        thread::spawn(move || provider_runtime.activate_module("BatchReceiptProvider"));
    provider_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the earlier provider activation must be blocked in build");

    let batch_runtime = runtime.clone();
    let batch_activation = thread::spawn(move || batch_runtime.activate_registered_modules());
    let handle = runtime.handle();
    let join_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let joined = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .waiter_count("BatchReceiptProvider", ModuleLifecycleCommand::Activate);
        if joined == 1 {
            break;
        }
        assert!(
            Instant::now() < join_deadline,
            "batch must join the earlier provider activation"
        );
        thread::yield_now();
    }

    release_provider_tx.send(()).unwrap();
    provider_activation.join().unwrap().unwrap();
    failure_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("batch must reacquire the provider before entering the later failing callback");

    let provider_joiner_runtime = runtime.clone();
    let provider_joiner =
        thread::spawn(move || provider_joiner_runtime.activate_module("BatchReceiptProvider"));
    let failure_receipt_runtime = runtime.clone();
    let failure_receipt =
        thread::spawn(move || failure_receipt_runtime.activate_module("BatchReceiptFailure"));

    let receipt_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let provider_waiters = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .waiter_count("BatchReceiptProvider", ModuleLifecycleCommand::Activate);
        if provider_waiters > 0 {
            break;
        }
        assert!(
            Instant::now() < receipt_deadline,
            "the public closure activation must join the batch provider receipt"
        );
        thread::yield_now();
    }

    release_failure_tx.send(()).unwrap();
    let batch_result = batch_activation.join().unwrap();
    let provider_join_result = provider_joiner.join().unwrap();
    let failure_join_result = failure_receipt.join().unwrap();
    match batch_result {
        Err(CoreError::MissingConfig(message)) => {
            assert_eq!(message, "batch.receipt.failure");
        }
        Err(CoreError::ModuleBatchActivationRollback { activation, .. }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref message) if message == "batch.receipt.failure"
            ));
        }
        other => panic!("expected typed joined-provider batch failure, got {other:?}"),
    }
    assert!(
        provider_join_result.is_ok(),
        "the already-running provider's joined successful receipt must remain successful"
    );
    assert!(matches!(
        failure_join_result,
        Err(CoreError::MissingConfig(message))
            if message == "batch.receipt.failure"
    ));
    assert_eq!(provider_build_calls.load(Ordering::SeqCst), 1);
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get("BatchReceiptProvider").unwrap().lifecycle,
        LifecycleState::Running
    );
    assert_eq!(
        modules.get("BatchReceiptFailure").unwrap().lifecycle,
        LifecycleState::Registered
    );
}

#[derive(Debug)]
struct ReentrantClosureMember;

impl ModuleLifecycle for ReentrantClosureMember {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .expect("activation callback must retain Core");
        let error = core
            .activate_module("ReentrantClosureProvider")
            .expect_err("same-thread reentry into a reserved closure member must fail fast");
        assert!(matches!(
            error,
            CoreError::ModuleLifecycleCommandReentrant { module, command }
                if module == "ReentrantClosureProvider" && command == "activate"
        ));
        Ok(())
    }
}

#[test]
fn activation_callback_reentry_into_another_reserved_closure_member_fails_fast() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new(
            "ReentrantClosureProvider",
            "reentrant closure",
        ))
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ReentrantClosureConsumer", "reentrant closure")
                .with_module_dependency(ModuleDependencySpec::named("ReentrantClosureProvider"))
                .with_lifecycle(Arc::new(ReentrantClosureMember)),
        )
        .unwrap();

    runtime.activate_module("ReentrantClosureConsumer").unwrap();
    runtime
        .deactivate_module("ReentrantClosureConsumer")
        .unwrap();
    runtime
        .deactivate_module("ReentrantClosureProvider")
        .unwrap();
}

#[test]
fn batch_owned_provider_waiter_receives_error_after_later_failure() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new(
            "BatchOwnedProvider",
            "batch-owned waiter",
        ))
        .unwrap();

    let (failure_started_tx, failure_started_rx) = mpsc::sync_channel(1);
    let (release_failure_tx, release_failure_rx) = mpsc::sync_channel(1);
    runtime
        .register_module(
            ModuleDescriptor::new("BatchOwnedFailure", "batch-owned waiter")
                .with_module_dependency(ModuleDependencySpec::named("BatchOwnedProvider"))
                .with_lifecycle(Arc::new(BlockingBatchFailure {
                    build_started: Mutex::new(Some(failure_started_tx)),
                    release_build: Mutex::new(release_failure_rx),
                })),
        )
        .unwrap();

    let batch_runtime = runtime.clone();
    let batch_activation = thread::spawn(move || batch_runtime.activate_registered_modules());
    failure_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the later batch member must hold the batch before it fails");

    let waiter_runtime = runtime.clone();
    let waiter = thread::spawn(move || waiter_runtime.activate_module("BatchOwnedProvider"));
    let handle = runtime.handle();
    let deadline = Instant::now() + Duration::from_secs(2);
    while handle
        .inner
        .lifecycle_coordinator
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .waiter_count("BatchOwnedProvider", ModuleLifecycleCommand::Activate)
        == 0
    {
        assert!(
            Instant::now() < deadline,
            "the provider waiter must join the batch transition"
        );
        thread::yield_now();
    }

    release_failure_tx
        .send(())
        .expect("release the later failing callback");
    let batch_result = batch_activation.join().unwrap();
    assert!(matches!(
        batch_result,
        Err(CoreError::MissingConfig(message))
            if message == "batch.receipt.failure"
    ));
    let waiter_result = waiter.join().unwrap();
    assert!(matches!(
        waiter_result,
        Err(CoreError::MissingConfig(message))
            if message == "batch.receipt.failure"
    ));

    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get("BatchOwnedProvider").unwrap().lifecycle,
        LifecycleState::Registered
    );
    assert_eq!(
        modules.get("BatchOwnedFailure").unwrap().lifecycle,
        LifecycleState::Registered
    );
    drop(modules);
    assert!(handle.active_module_shutdown_order().is_empty());
}

#[derive(Debug)]
struct LazyBuildFailure {
    service_name: String,
}

impl ModuleLifecycle for LazyBuildFailure {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .expect("lazy build callback must retain Core");
        let _instance = core.resolve_manager::<LazyBuildService>(&self.service_name)?;
        Err(CoreError::MissingConfig(
            "batch.lazy.build.failure".to_owned(),
        ))
    }
}

#[derive(Debug)]
struct ResolveLaterLazyThenFail {
    service_name: String,
    fail_once: Arc<AtomicBool>,
}

impl ModuleLifecycle for ResolveLaterLazyThenFail {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        if !self.fail_once.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let core = context
            .core
            .upgrade()
            .expect("later lazy build callback must retain Core");
        let _instance = core.resolve_manager::<LazyBuildService>(&self.service_name)?;
        Err(CoreError::MissingConfig(
            "batch.lazy.later.build.failure".to_owned(),
        ))
    }
}

#[derive(Debug)]
struct LazyBuildService {
    dropped: mpsc::Sender<()>,
}

impl Drop for LazyBuildService {
    fn drop(&mut self) {
        let _ = self.dropped.send(());
    }
}

#[test]
fn batch_failure_rolls_back_lazy_service_created_during_build() {
    let runtime = CoreRuntime::new();
    let module = "BatchLazyBuildFailure";
    let service_name = RegistryName::from_parts(module, ServiceKind::Manager, "Lazy");
    let service_key = service_name.as_str().to_owned();
    let (dropped_tx, dropped_rx) = mpsc::channel();
    let factory_dropped = dropped_tx.clone();
    runtime
        .register_module(
            ModuleDescriptor::new(module, "lazy build failure")
                .with_lifecycle(Arc::new(LazyBuildFailure {
                    service_name: service_key.clone(),
                }))
                .with_manager(ManagerDescriptor::new(
                    service_name,
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(LazyBuildService {
                            dropped: factory_dropped.clone(),
                        }) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    assert!(runtime.activate_registered_modules().is_err());
    assert!(dropped_rx.recv_timeout(Duration::from_secs(2)).is_ok());

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get(module).unwrap().lifecycle,
        LifecycleState::Registered
    );
    drop(modules);
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service_key.as_str()).unwrap();
    assert_eq!(entry.lifecycle, LifecycleState::Registered);
    assert!(entry.instance.is_none());
    assert!(!entry.admission_open);
}

#[test]
fn batch_failure_resets_unstarted_materialized_lazy_member_before_retry() {
    let runtime = CoreRuntime::new();
    let failing_module = "BatchLazyLaterFailure";
    let later_module = "BatchLazyLaterMember";
    let service = RegistryName::from_parts(later_module, ServiceKind::Manager, "Lazy");
    let service_key = service.as_str().to_owned();
    let (dropped_tx, dropped_rx) = mpsc::channel();
    let factory_dropped = dropped_tx.clone();
    runtime
        .register_module(
            ModuleDescriptor::new(failing_module, "later lazy rollback").with_lifecycle(Arc::new(
                ResolveLaterLazyThenFail {
                    service_name: service_key.clone(),
                    fail_once: Arc::new(AtomicBool::new(true)),
                },
            )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new(later_module, "later lazy rollback")
                .with_module_dependency(ModuleDependencySpec::named(failing_module))
                .with_manager(ManagerDescriptor::new(
                    service,
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(LazyBuildService {
                            dropped: factory_dropped.clone(),
                        }) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    assert!(matches!(
        runtime.activate_registered_modules(),
        Err(CoreError::MissingConfig(message))
            if message == "batch.lazy.later.build.failure"
    ));
    assert!(dropped_rx.recv_timeout(Duration::from_secs(2)).is_ok());

    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get(failing_module).unwrap().lifecycle,
            LifecycleState::Registered
        );
        assert_eq!(
            modules.get(later_module).unwrap().lifecycle,
            LifecycleState::Registered
        );
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(service_key.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime
        .activate_registered_modules()
        .expect("retry must rebuild the later lazy member after compensation");
    let handle_guard = runtime
        .resolve_manager_handle::<LazyBuildService>(service_key.as_str())
        .expect("retry must reopen the lazy service admission")
        .enter()
        .expect("retry-created lazy service must admit guarded calls");
    drop(handle_guard);
}

#[derive(Debug)]
struct PanicOnBatchModule {
    panic_module: String,
}

impl RuntimeModuleLifecycleObserver for PanicOnBatchModule {
    fn runtime_module_activated(&self, module_name: &str) {
        if module_name == self.panic_module {
            panic!("post-finish batch notification failed");
        }
    }

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Ok(())
    }
}

#[derive(Debug)]
struct CensusThenPanicOnBatchModule {
    core: CoreWeak,
    panic_module: String,
    observed: Mutex<Option<mpsc::SyncSender<(LifecycleState, bool)>>>,
}

impl RuntimeModuleLifecycleObserver for CensusThenPanicOnBatchModule {
    fn runtime_module_activated(&self, module_name: &str) {
        if module_name != self.panic_module {
            return;
        }
        let core = self
            .core
            .upgrade()
            .expect("census observer must retain Core");
        let lifecycle = core
            .inner
            .modules
            .lock()
            .unwrap()
            .get(module_name)
            .expect("notified module must remain registered")
            .lifecycle;
        let active = core
            .active_module_shutdown_order()
            .iter()
            .any(|active| active == module_name);
        if let Some(sender) = self.observed.lock().unwrap().take() {
            sender.send((lifecycle, active)).unwrap();
        }
        panic!("post-finish batch notification failed after census");
    }

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Ok(())
    }
}

fn register_batch_census_module(runtime: &CoreRuntime, module: &str) -> String {
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "Census");
    let key = service.as_str().to_owned();
    runtime
        .register_module(ModuleDescriptor::new(module, "batch census").with_manager(
            ManagerDescriptor::new(
                service,
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
            ),
        ))
        .unwrap();
    key
}

#[test]
fn batch_post_finish_notification_failure_restores_exact_module_service_census() {
    let runtime = CoreRuntime::new();
    let provider_service = register_batch_census_module(&runtime, "BatchCensusProvider");
    let failing_service_name =
        RegistryName::from_parts("BatchCensusFailure", ServiceKind::Manager, "Census");
    let failing_service = failing_service_name.as_str().to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new("BatchCensusFailure", "batch census")
                .with_module_dependency(ModuleDependencySpec::named("BatchCensusProvider"))
                .with_manager(ManagerDescriptor::new(
                    failing_service_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
                )),
        )
        .unwrap();
    let (observed_tx, observed_rx) = mpsc::sync_channel(1);
    runtime.install_runtime_module_lifecycle_observer(Arc::new(CensusThenPanicOnBatchModule {
        core: runtime.weak(),
        panic_module: "BatchCensusFailure".to_owned(),
        observed: Mutex::new(Some(observed_tx)),
    }));

    let activation_result = runtime.activate_registered_modules();
    match activation_result {
        Err(CoreError::ModuleLifecycleCallbackPanicked { module, command }) => {
            assert_eq!(module, "BatchCensusFailure");
            assert_eq!(command, "activate");
        }
        Err(CoreError::ModuleBatchActivationRollback { activation, .. }) => {
            assert!(matches!(
                *activation,
                CoreError::ModuleLifecycleCallbackPanicked {
                    ref module,
                    command: "activate",
                } if module == "BatchCensusFailure"
            ));
        }
        other => panic!("expected typed observer notification failure, got {other:?}"),
    }
    assert_eq!(
        observed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        (LifecycleState::Running, true)
    );
    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module in ["BatchCensusProvider", "BatchCensusFailure"] {
        assert_eq!(
            modules.get(module).unwrap().lifecycle,
            LifecycleState::Registered
        );
    }
    drop(modules);
    assert!(handle.active_module_shutdown_order().is_empty());
    let services = handle.inner.services.lock().unwrap();
    for service in [provider_service, failing_service] {
        let entry = services.get(service.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }
}

#[derive(Debug)]
struct CensusThenPanicOnceObserver {
    core: CoreWeak,
    module: String,
    snapshots: Arc<Mutex<Vec<(LifecycleState, bool)>>>,
    panic_next: AtomicBool,
}

impl RuntimeModuleLifecycleObserver for CensusThenPanicOnceObserver {
    fn runtime_module_activated(&self, module_name: &str) {
        if module_name != self.module {
            return;
        }
        let core = self
            .core
            .upgrade()
            .expect("retry census observer must retain Core");
        let lifecycle = core
            .inner
            .modules
            .lock()
            .unwrap()
            .get(module_name)
            .expect("retry-notified module must remain registered")
            .lifecycle;
        let active = core
            .active_module_shutdown_order()
            .iter()
            .any(|active| active == module_name);
        self.snapshots.lock().unwrap().push((lifecycle, active));
        if self.panic_next.swap(false, Ordering::AcqRel) {
            panic!("one-shot post-finish notification failure");
        }
    }

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Ok(())
    }
}

#[test]
fn post_finish_notification_failure_retry_publishes_running_census_again() {
    let runtime = CoreRuntime::new();
    let service_key = register_batch_census_module(&runtime, "RetryCensusModule");
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    runtime.install_runtime_module_lifecycle_observer(Arc::new(CensusThenPanicOnceObserver {
        core: runtime.weak(),
        module: "RetryCensusModule".to_owned(),
        snapshots: Arc::clone(&snapshots),
        panic_next: AtomicBool::new(true),
    }));

    assert!(runtime.activate_module("RetryCensusModule").is_err());
    runtime
        .activate_module("RetryCensusModule")
        .expect("a post-finish observer failure must leave a retryable module");

    assert_eq!(
        *snapshots.lock().unwrap(),
        vec![
            (LifecycleState::Running, true),
            (LifecycleState::Running, true)
        ]
    );
    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get("RetryCensusModule").unwrap().lifecycle,
        LifecycleState::Running
    );
    assert!(handle
        .active_module_shutdown_order()
        .iter()
        .any(|module| module == "RetryCensusModule"));
    drop(modules);
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(service_key.as_str()).unwrap();
    assert_eq!(entry.lifecycle, LifecycleState::Running);
    assert!(entry.instance.is_some());
}

struct HeldServiceCallObserver {
    core: CoreWeak,
    service_name: String,
    entered: Mutex<Option<mpsc::SyncSender<()>>>,
    release: Arc<Mutex<mpsc::Receiver<()>>>,
    callback_entered: Mutex<Option<mpsc::SyncSender<()>>>,
    callback_observed: Mutex<mpsc::Receiver<()>>,
    panic_module: String,
    worker: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    worker_done: Arc<Mutex<Option<mpsc::SyncSender<()>>>>,
}

impl std::fmt::Debug for HeldServiceCallObserver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("HeldServiceCallObserver").finish()
    }
}

impl RuntimeModuleLifecycleObserver for HeldServiceCallObserver {
    fn runtime_module_activated(&self, module_name: &str) {
        if module_name == self.panic_module {
            panic!("held-call batch notification failed");
        }
        if module_name != "HeldCallProvider" {
            return;
        }
        let core = self.core.clone();
        let service_name = self.service_name.clone();
        let release = Arc::clone(&self.release);
        let worker_done = Arc::clone(&self.worker_done);
        let entered = self.entered.lock().unwrap().take();
        let worker = thread::spawn(move || {
            let handle = core
                .upgrade()
                .expect("held-call observer must retain Core")
                .resolve_manager_handle::<HeldCallService>(service_name.as_str())
                .expect("the provider service must be running before notification");
            let _call = handle.enter().expect("the provider call must be admitted");
            if let Some(entered) = entered {
                entered.send(()).unwrap();
            }
            let _ = release.lock().unwrap().recv();
            if let Some(done) = worker_done.lock().unwrap().take() {
                done.send(()).unwrap();
            }
        });
        *self.worker.lock().unwrap() = Some(worker);
        self.callback_observed
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(2))
            .expect("observer must wait for the held call to enter");
        if let Some(callback_entered) = self.callback_entered.lock().unwrap().take() {
            callback_entered.send(()).unwrap();
        }
    }

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Ok(())
    }
}

#[derive(Debug)]
struct HeldCallService;

#[test]
fn batch_rollback_rejects_held_service_call_until_retry_drain() {
    let runtime = CoreRuntime::new();
    let provider = "HeldCallProvider";
    let service = RegistryName::from_parts(provider, ServiceKind::Manager, "Held");
    let service_key = service.as_str().to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new(provider, "held call provider").with_manager(
                ManagerDescriptor::new(
                    service,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(HeldCallService) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime
        .register_module(ModuleDescriptor::new(
            "HeldCallFailure",
            "held call failure",
        ))
        .unwrap();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (callback_entered_tx, callback_entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::channel();
    let worker_slot = Arc::new(Mutex::new(None));
    let (worker_done_tx, worker_done_rx) = mpsc::sync_channel(1);
    runtime.install_runtime_module_lifecycle_observer(Arc::new(HeldServiceCallObserver {
        core: runtime.weak(),
        service_name: service_key.clone(),
        entered: Mutex::new(Some(entered_tx)),
        release: Arc::new(Mutex::new(release_rx)),
        callback_entered: Mutex::new(Some(callback_entered_tx)),
        callback_observed: Mutex::new(entered_rx),
        panic_module: "HeldCallFailure".to_owned(),
        worker: Arc::clone(&worker_slot),
        worker_done: Arc::new(Mutex::new(Some(worker_done_tx))),
    }));

    let batch_runtime = runtime.clone();
    let (batch_result_tx, batch_result_rx) = mpsc::channel();
    let batch = thread::spawn(move || {
        let result = batch_runtime.activate_registered_modules();
        batch_result_tx.send(result).unwrap();
    });
    callback_entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the observer must publish the held call before the later failure");
    let batch_result = batch_result_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("zero-budget rollback must reject the held call");
    match batch_result {
        Err(CoreError::ModuleBatchActivationRollback {
            activation,
            cleanup_failures,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::ModuleLifecycleCallbackPanicked {
                    ref module,
                    command: "activate",
                } if module == "HeldCallFailure"
            ));
            assert!(cleanup_failures.iter().any(|(module, error)| {
                module == provider
                    && matches!(
                        error,
                        CoreError::ServiceCallDrainTimeout {
                            module: error_module,
                            in_flight_calls: 1,
                            ..
                        } if error_module == provider
                    )
            }));
        }
        other => panic!("expected typed held-call batch rollback, got {other:?}"),
    }

    let handle = runtime.handle();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get(provider)
            .unwrap()
            .lifecycle,
        LifecycleState::Stopping
    );
    assert!(runtime
        .deactivate_module_with_drain_timeout(provider, Duration::ZERO)
        .is_err());
    assert_eq!(
        handle
            .inner
            .services
            .lock()
            .unwrap()
            .get(service_key.as_str())
            .unwrap()
            .in_flight_calls,
        1
    );
    release_tx.send(()).unwrap();
    worker_done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("held call worker must terminate after release");
    worker_slot
        .lock()
        .unwrap()
        .take()
        .expect("observer must retain the held worker for a bounded join")
        .join()
        .expect("held call worker must not panic");
    batch.join().unwrap();
    runtime
        .deactivate_module_with_drain_timeout(provider, Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get(provider)
            .unwrap()
            .lifecycle,
        LifecycleState::Unloaded
    );
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(service_key.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
        assert_eq!(entry.in_flight_calls, 0);
    }
}

#[derive(Debug)]
struct RetryCleanup {
    failed: Arc<AtomicBool>,
}

impl ModuleLifecycle for RetryCleanup {
    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        if self.failed.swap(false, Ordering::AcqRel) {
            Err(CoreError::MissingConfig("retry.batch.cleanup".to_owned()))
        } else {
            Ok(())
        }
    }
}

#[test]
fn batch_failed_cleanup_retry_preserves_owner_until_terminal_success() {
    let runtime = CoreRuntime::new();
    let module = "BatchRetryCleanup";
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "Owner");
    let service_key = service.as_str().to_owned();
    let failed = Arc::new(AtomicBool::new(true));
    runtime
        .register_module(
            ModuleDescriptor::new(module, "retry cleanup")
                .with_lifecycle(Arc::new(RetryCleanup {
                    failed: Arc::clone(&failed),
                }))
                .with_manager(ManagerDescriptor::new(
                    service,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime.install_runtime_module_lifecycle_observer(Arc::new(PanicOnBatchModule {
        panic_module: module.to_owned(),
    }));

    assert!(runtime.activate_registered_modules().is_err());
    let handle = runtime.handle();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get(module)
            .unwrap()
            .lifecycle,
        LifecycleState::Stopping
    );
    let services = handle.inner.services.lock().unwrap();
    let owner = services.get(service_key.as_str()).unwrap();
    assert!(owner.instance.is_some());
    assert!(!owner.admission_open);
    drop(services);
    assert!(!handle.active_module_shutdown_order().is_empty());

    runtime
        .deactivate_module_with_drain_timeout(module, Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get(module)
            .unwrap()
            .lifecycle,
        LifecycleState::Unloaded
    );
    let services = handle.inner.services.lock().unwrap();
    assert_eq!(
        services.get(service_key.as_str()).unwrap().lifecycle,
        LifecycleState::Unloaded
    );
    assert!(services
        .get(service_key.as_str())
        .unwrap()
        .instance
        .is_none());
}

#[derive(Debug)]
struct SequencedCleanup {
    name: &'static str,
    failed: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl ModuleLifecycle for SequencedCleanup {
    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.events.lock().unwrap().push(self.name);
        if self.failed.swap(false, Ordering::AcqRel) {
            Err(CoreError::MissingConfig(format!(
                "batch.cleanup.{}",
                self.name
            )))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
struct LateBatchBuildFailure;

impl ModuleLifecycle for LateBatchBuildFailure {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        Err(CoreError::MissingConfig(
            "batch.late.build.failure".to_owned(),
        ))
    }
}

#[derive(Debug)]
struct DropCounterService {
    drops: Arc<AtomicUsize>,
}

impl Drop for DropCounterService {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn register_batch_cleanup_module(
    runtime: &CoreRuntime,
    module: &'static str,
    dependency: Option<&'static str>,
    lifecycle: Arc<SequencedCleanup>,
    drops: Option<Arc<AtomicUsize>>,
) -> String {
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "Owned");
    let service_key = service.as_str().to_owned();
    let mut descriptor = ModuleDescriptor::new(module, "batch cleanup compensation")
        .with_lifecycle(lifecycle)
        .with_manager(ManagerDescriptor::new(
            service,
            StartupMode::Immediate,
            Vec::new(),
            Arc::new(move |_| {
                if let Some(drops) = drops.as_ref() {
                    return Ok(Arc::new(DropCounterService {
                        drops: Arc::clone(drops),
                    }) as ServiceObject);
                }
                Ok(Arc::new(()) as ServiceObject)
            }),
        ));
    if let Some(dependency) = dependency {
        descriptor = descriptor.with_module_dependency(ModuleDependencySpec::named(dependency));
    }
    runtime.register_module(descriptor).unwrap();
    service_key
}

#[test]
fn batch_cleanup_failure_retains_provider_until_dependent_retry_then_cleans_in_order() {
    let runtime = CoreRuntime::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let provider_calls = Arc::new(AtomicUsize::new(0));
    let dependent_calls = Arc::new(AtomicUsize::new(0));
    let provider_service = register_batch_cleanup_module(
        &runtime,
        "CompensationProvider",
        None,
        Arc::new(SequencedCleanup {
            name: "provider",
            failed: Arc::new(AtomicBool::new(false)),
            calls: Arc::clone(&provider_calls),
            events: Arc::clone(&events),
        }),
        None,
    );
    let dependent_service = register_batch_cleanup_module(
        &runtime,
        "CompensationDependent",
        Some("CompensationProvider"),
        Arc::new(SequencedCleanup {
            name: "dependent",
            failed: Arc::new(AtomicBool::new(true)),
            calls: Arc::clone(&dependent_calls),
            events: Arc::clone(&events),
        }),
        None,
    );
    let late = ModuleDescriptor::new("CompensationLateFailure", "batch cleanup compensation")
        .with_module_dependency(ModuleDependencySpec::named("CompensationDependent"))
        .with_lifecycle(Arc::new(LateBatchBuildFailure));
    runtime.register_module(late).unwrap();

    match runtime.activate_registered_modules() {
        Err(CoreError::ModuleBatchActivationRollback {
            activation,
            cleanup_failures,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref message) if message == "batch.late.build.failure"
            ));
            assert!(cleanup_failures.iter().any(|(module, error)| {
                module == "CompensationDependent"
                    && matches!(
                        error,
                        CoreError::MissingConfig(message)
                            if message == "batch.cleanup.dependent"
                    )
            }));
        }
        other => panic!("expected typed provider/dependent compensation, got {other:?}"),
    }
    let handle = runtime.handle();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get("CompensationProvider")
            .unwrap()
            .lifecycle,
        LifecycleState::Stopping
    );
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);
    assert_eq!(dependent_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        handle
            .inner
            .services
            .lock()
            .unwrap()
            .get(provider_service.as_str())
            .unwrap()
            .instance
            .is_some(),
        true
    );
    assert!(
        !handle
            .inner
            .services
            .lock()
            .unwrap()
            .get(dependent_service.as_str())
            .unwrap()
            .admission_open
    );

    runtime
        .deactivate_module_with_drain_timeout("CompensationDependent", Duration::from_secs(1))
        .unwrap();
    runtime
        .deactivate_module_with_drain_timeout("CompensationProvider", Duration::from_secs(1))
        .unwrap();
    assert_eq!(events.lock().unwrap().as_slice(), ["dependent", "provider"]);
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get("CompensationDependent").unwrap().lifecycle,
            LifecycleState::Unloaded
        );
        assert_eq!(
            modules.get("CompensationProvider").unwrap().lifecycle,
            LifecycleState::Unloaded
        );
    }
    {
        let services = handle.inner.services.lock().unwrap();
        for service in [dependent_service.as_str(), provider_service.as_str()] {
            let entry = services.get(service).unwrap();
            assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
            assert!(entry.instance.is_none());
            assert!(!entry.admission_open);
        }
    }
}

#[test]
fn batch_cleanup_restores_successful_independent_member_without_replaying_drop() {
    let runtime = CoreRuntime::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let independent_calls = Arc::new(AtomicUsize::new(0));
    let failing_calls = Arc::new(AtomicUsize::new(0));
    let independent_drops = Arc::new(AtomicUsize::new(0));
    let independent_service = register_batch_cleanup_module(
        &runtime,
        "CompensationIndependent",
        None,
        Arc::new(SequencedCleanup {
            name: "independent",
            failed: Arc::new(AtomicBool::new(false)),
            calls: Arc::clone(&independent_calls),
            events: Arc::clone(&events),
        }),
        Some(Arc::clone(&independent_drops)),
    );
    let sibling_service = register_batch_cleanup_module(
        &runtime,
        "CompensationSiblingFailure",
        None,
        Arc::new(SequencedCleanup {
            name: "sibling",
            failed: Arc::new(AtomicBool::new(true)),
            calls: Arc::clone(&failing_calls),
            events: Arc::clone(&events),
        }),
        None,
    );
    runtime
        .register_module(
            ModuleDescriptor::new("CompensationIndependentLate", "batch cleanup compensation")
                .with_lifecycle(Arc::new(LateBatchBuildFailure)),
        )
        .unwrap();

    match runtime.activate_registered_modules() {
        Err(CoreError::ModuleBatchActivationRollback {
            activation,
            cleanup_failures,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref message) if message == "batch.late.build.failure"
            ));
            assert!(cleanup_failures.iter().any(|(module, error)| {
                module == "CompensationSiblingFailure"
                    && matches!(
                        error,
                        CoreError::MissingConfig(message)
                            if message == "batch.cleanup.sibling"
                    )
            }));
        }
        other => panic!("expected typed independent-member compensation, got {other:?}"),
    }
    let handle = runtime.handle();
    assert_eq!(independent_calls.load(Ordering::SeqCst), 1);
    assert_eq!(failing_calls.load(Ordering::SeqCst), 1);
    assert_eq!(independent_drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get("CompensationIndependent")
            .unwrap()
            .lifecycle,
        LifecycleState::Registered
    );
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services
            .get(independent_service.as_str())
            .expect("independent service remains registered");
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime
        .deactivate_module_with_drain_timeout("CompensationSiblingFailure", Duration::from_secs(1))
        .unwrap();
    assert_eq!(independent_calls.load(Ordering::SeqCst), 1);
    assert_eq!(independent_drops.load(Ordering::SeqCst), 1);
    assert_eq!(failing_calls.load(Ordering::SeqCst), 2);
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get("CompensationSiblingFailure").unwrap().lifecycle,
            LifecycleState::Unloaded
        );
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(sibling_service.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime
        .activate_module("CompensationIndependent")
        .expect("successful independent cleanup must leave a retryable module");
    assert_eq!(independent_calls.load(Ordering::SeqCst), 1);
    assert_eq!(independent_drops.load(Ordering::SeqCst), 1);
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules.get("CompensationIndependent").unwrap().lifecycle,
        LifecycleState::Running
    );
    assert!(handle
        .active_module_shutdown_order()
        .iter()
        .any(|module| module == "CompensationIndependent"));
    drop(modules);
    let services = handle.inner.services.lock().unwrap();
    let entry = services.get(independent_service.as_str()).unwrap();
    assert_eq!(entry.lifecycle, LifecycleState::Running);
    assert!(entry.instance.is_some());
}

#[derive(Debug)]
struct PanicOnBatchServiceDrop {
    drop_started: Arc<AtomicBool>,
    drop_sequence: Arc<Mutex<Vec<&'static str>>>,
}

impl Drop for PanicOnBatchServiceDrop {
    fn drop(&mut self) {
        self.drop_started.store(true, Ordering::SeqCst);
        self.drop_sequence.lock().unwrap().push("drop-before-panic");
        panic!("intentional batch service destructor panic");
    }
}

#[test]
fn batch_cleanup_receipt_survives_panic_on_drop_and_retry_skips_callback() {
    let runtime = CoreRuntime::new();
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_events = Arc::new(Mutex::new(Vec::new()));
    let drop_started = Arc::new(AtomicBool::new(false));
    let drop_sequence = Arc::new(Mutex::new(Vec::new()));
    let module = "CompensationPanicDrop";
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "Owned");
    let service_key = service.as_str().to_owned();
    runtime
        .register_module(
            ModuleDescriptor::new(module, "batch cleanup panic drop")
                .with_lifecycle(Arc::new(SequencedCleanup {
                    name: "panic-drop",
                    failed: Arc::new(AtomicBool::new(false)),
                    calls: Arc::clone(&cleanup_calls),
                    events: Arc::clone(&cleanup_events),
                }))
                .with_manager(ManagerDescriptor::new(
                    service,
                    StartupMode::Immediate,
                    Vec::new(),
                    {
                        let drop_started = Arc::clone(&drop_started);
                        let drop_sequence = Arc::clone(&drop_sequence);
                        Arc::new(move |_| {
                            Ok(Arc::new(PanicOnBatchServiceDrop {
                                drop_started: Arc::clone(&drop_started),
                                drop_sequence: Arc::clone(&drop_sequence),
                            }) as ServiceObject)
                        })
                    },
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("CompensationPanicDropLate", "batch cleanup panic drop")
                .with_module_dependency(ModuleDependencySpec::named(module))
                .with_lifecycle(Arc::new(LateBatchBuildFailure)),
        )
        .unwrap();

    let batch_result = runtime.activate_registered_modules();
    match batch_result {
        Err(CoreError::ModuleBatchActivationRollback {
            activation,
            cleanup_failures,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref key) if key == "batch.late.build.failure"
            ));
            assert!(cleanup_failures.iter().any(|(failed_module, error)| {
                failed_module == module
                    && matches!(
                        error,
                        CoreError::ServiceRetirementPanicked {
                            module: error_module,
                            service: error_service,
                            command,
                        } if error_module == module
                            && error_service == &service_key
                            && *command == "activate"
                    )
            }));
        }
        other => panic!("expected typed panic-drop batch rollback, got {other:?}"),
    }
    assert!(drop_started.load(Ordering::SeqCst));
    assert_eq!(
        drop_sequence.lock().unwrap().as_slice(),
        ["drop-before-panic"]
    );
    let handle = runtime.handle();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    assert_eq!(cleanup_events.lock().unwrap().as_slice(), ["panic-drop"]);
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(modules.len(), 2);
        assert_eq!(
            modules.get(module).unwrap().lifecycle,
            LifecycleState::Stopping
        );
        assert!(modules.get(module).unwrap().cleanup_completed);
        assert_eq!(
            modules.get("CompensationPanicDropLate").unwrap().lifecycle,
            LifecycleState::Registered
        );
    }
    {
        let services = handle.inner.services.lock().unwrap();
        assert_eq!(services.len(), 1);
        let entry = services.get(service_key.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }
    assert_eq!(
        handle.active_module_shutdown_order(),
        vec![module.to_owned()]
    );

    runtime
        .deactivate_module_with_drain_timeout(module, Duration::from_secs(1))
        .unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get(module).unwrap().lifecycle,
            LifecycleState::Unloaded
        );
        assert!(!modules.get(module).unwrap().cleanup_completed);
    }
    {
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(service_key.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }
    assert!(handle.active_module_shutdown_order().is_empty());
}

#[derive(Debug)]
struct GatedFinish {
    fail: Arc<AtomicBool>,
    error_key: &'static str,
    cleanup_calls: Arc<AtomicUsize>,
}

impl ModuleLifecycle for GatedFinish {
    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        if self.fail.load(Ordering::Acquire) {
            Err(CoreError::MissingConfig(self.error_key.to_owned()))
        } else {
            Ok(())
        }
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Debug)]
struct PanicOnActivationServiceDrop {
    panic_on_drop: Arc<AtomicBool>,
    drop_started: Arc<AtomicBool>,
    drop_sequence: Arc<Mutex<Vec<&'static str>>>,
}

impl Drop for PanicOnActivationServiceDrop {
    fn drop(&mut self) {
        self.drop_started.store(true, Ordering::SeqCst);
        self.drop_sequence.lock().unwrap().push("drop-before-panic");
        if self.panic_on_drop.swap(false, Ordering::SeqCst) {
            panic!("intentional activation service destructor panic");
        }
        self.drop_sequence.lock().unwrap().push("drop-complete");
    }
}

fn register_activation_drop_probe(
    runtime: &CoreRuntime,
    module: &'static str,
    error_key: &'static str,
    fail: Arc<AtomicBool>,
    panic_on_drop: Arc<AtomicBool>,
    drop_started: Arc<AtomicBool>,
    drop_sequence: Arc<Mutex<Vec<&'static str>>>,
    cleanup_calls: Arc<AtomicUsize>,
) -> RegistryName {
    let service = RegistryName::from_parts(module, ServiceKind::Manager, "PanicDrop");
    let service_factory = {
        let panic_on_drop = Arc::clone(&panic_on_drop);
        let drop_started = Arc::clone(&drop_started);
        let drop_sequence = Arc::clone(&drop_sequence);
        Arc::new(move |_: &CoreWeak| {
            Ok(Arc::new(PanicOnActivationServiceDrop {
                panic_on_drop: Arc::clone(&panic_on_drop),
                drop_started: Arc::clone(&drop_started),
                drop_sequence: Arc::clone(&drop_sequence),
            }) as ServiceObject)
        })
    };
    runtime
        .register_module(
            ModuleDescriptor::new(module, "activation retirement panic")
                .with_lifecycle(Arc::new(GatedFinish {
                    fail,
                    error_key,
                    cleanup_calls,
                }))
                .with_manager(ManagerDescriptor::new(
                    service.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    service_factory,
                )),
        )
        .unwrap();
    service
}

#[test]
fn single_activation_retirement_panic_is_typed_and_retryable() {
    let runtime = CoreRuntime::new();
    let fail = Arc::new(AtomicBool::new(true));
    let panic_on_drop = Arc::new(AtomicBool::new(true));
    let drop_started = Arc::new(AtomicBool::new(false));
    let drop_sequence = Arc::new(Mutex::new(Vec::new()));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let module = "SingleActivationRetirementPanic";
    let service = register_activation_drop_probe(
        &runtime,
        module,
        "single.activation.finish",
        Arc::clone(&fail),
        Arc::clone(&panic_on_drop),
        Arc::clone(&drop_started),
        Arc::clone(&drop_sequence),
        Arc::clone(&cleanup_calls),
    );

    let activation = runtime.activate_module(module);
    match activation {
        Err(CoreError::ModuleActivationRollback {
            activation,
            cleanup,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref key) if key == "single.activation.finish"
            ));
            assert!(matches!(
                *cleanup,
                CoreError::ServiceRetirementPanicked {
                    module: failed_module,
                    service: failed_service,
                    command,
                } if failed_module == module
                    && failed_service == service.to_string()
                    && command == "activate"
            ));
        }
        other => panic!("expected typed single activation rollback, got {other:?}"),
    }
    assert!(drop_started.load(Ordering::Acquire));
    assert_eq!(
        drop_sequence.lock().unwrap().as_slice(),
        ["drop-before-panic"]
    );
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    {
        let handle = runtime.handle();
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get(module).unwrap().lifecycle,
            LifecycleState::Stopping
        );
        assert!(modules.get(module).unwrap().cleanup_completed);
        assert_eq!(
            handle.active_module_shutdown_order(),
            vec![module.to_owned()]
        );
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(service.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Registered);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    fail.store(false, Ordering::Release);
    runtime.activate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        drop_sequence.lock().unwrap().as_slice(),
        ["drop-before-panic", "drop-before-panic", "drop-complete",]
    );
}

#[test]
fn failed_reactivation_retirement_panic_is_typed_and_retryable() {
    let runtime = CoreRuntime::new();
    let fail = Arc::new(AtomicBool::new(false));
    let panic_on_drop = Arc::new(AtomicBool::new(false));
    let drop_started = Arc::new(AtomicBool::new(false));
    let drop_sequence = Arc::new(Mutex::new(Vec::new()));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let module = "FailedReactivationRetirementPanic";
    let service = register_activation_drop_probe(
        &runtime,
        module,
        "failed.reactivation.finish",
        Arc::clone(&fail),
        Arc::clone(&panic_on_drop),
        Arc::clone(&drop_started),
        Arc::clone(&drop_sequence),
        Arc::clone(&cleanup_calls),
    );

    runtime.activate_module(module).unwrap();
    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    panic_on_drop.store(true, Ordering::Release);
    fail.store(true, Ordering::Release);

    let activation = runtime.activate_module(module);
    match activation {
        Err(CoreError::ModuleActivationRollback {
            activation,
            cleanup,
        }) => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref key) if key == "failed.reactivation.finish"
            ));
            assert!(matches!(
                *cleanup,
                CoreError::ServiceRetirementPanicked {
                    module: failed_module,
                    service: failed_service,
                    command,
                } if failed_module == module
                    && failed_service == service.to_string()
                    && command == "activate"
            ));
        }
        other => panic!("expected typed failed-reactivation rollback, got {other:?}"),
    }
    assert!(drop_started.load(Ordering::Acquire));
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 2);
    {
        let handle = runtime.handle();
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get(module).unwrap().lifecycle,
            LifecycleState::Stopping
        );
        assert!(modules.get(module).unwrap().cleanup_completed);
        let services = handle.inner.services.lock().unwrap();
        let entry = services.get(service.as_str()).unwrap();
        assert_eq!(entry.lifecycle, LifecycleState::Unloaded);
        assert!(entry.instance.is_none());
        assert!(!entry.admission_open);
    }

    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 2);
    fail.store(false, Ordering::Release);
    runtime.activate_module(module).unwrap();
    runtime.deactivate_module(module).unwrap();
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        drop_sequence.lock().unwrap().as_slice(),
        [
            "drop-before-panic",
            "drop-complete",
            "drop-before-panic",
            "drop-before-panic",
            "drop-complete",
        ]
    );
}
