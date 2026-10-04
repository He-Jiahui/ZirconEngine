use super::*;

mod aggregate;

fn shutdown_fixture() -> (CoreRuntime, Arc<Mutex<Vec<String>>>) {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    for (name, level, dependency, fails_once) in [
        ("ShutdownIndependent", InitLevel::Kernel, None, false),
        ("ShutdownProvider", InitLevel::Kernel, None, false),
        (
            "ShutdownDependent",
            InitLevel::Services,
            Some("ShutdownProvider"),
            true,
        ),
    ] {
        let lifecycle = RecordingLifecycle::new(Arc::clone(&calls));
        let lifecycle = if fails_once {
            lifecycle.fail_cleanup_once()
        } else {
            lifecycle
        };
        let mut descriptor = ModuleDescriptor::new(name, "dependency-safe shutdown")
            .with_init_level(level)
            .with_lifecycle(Arc::new(lifecycle));
        if let Some(dependency) = dependency {
            descriptor = descriptor.with_module_dependency(ModuleDependencySpec::named(dependency));
        }
        runtime.register_module(descriptor).unwrap();
    }
    runtime.activate_registered_modules().unwrap();
    (runtime, calls)
}

#[test]
fn shutdown_failure_keeps_stopping_dependents_and_continues_independent_modules() {
    let (runtime, calls) = shutdown_fixture();
    let report = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(!report.is_complete());
    assert_eq!(
        report.attempted,
        vec![
            "ShutdownDependent",
            "ShutdownProvider",
            "ShutdownIndependent"
        ]
    );
    assert_eq!(report.completed, vec!["ShutdownIndependent"]);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, "ShutdownDependent");
    assert!(matches!(&report.failed[0].1, CoreError::MissingConfig(_)));
    assert_eq!(report.blocked.len(), 1);
    assert_eq!(report.blocked[0].0, "ShutdownProvider");
    assert!(
        matches!(&report.blocked[0].1, CoreError::ModuleUnloadBlocked { dependents, .. }
        if dependents == &vec!["ShutdownDependent".to_owned()])
    );
    assert!(report.not_attempted.is_empty());
    assert!(!report.deadline_exhausted);
    let handle = runtime.handle();
    {
        let modules = handle.inner.modules.lock().unwrap();
        assert_eq!(
            modules.get("ShutdownDependent").unwrap().lifecycle,
            LifecycleState::Stopping
        );
        assert_eq!(
            modules.get("ShutdownProvider").unwrap().lifecycle,
            LifecycleState::Running
        );
        assert_eq!(
            modules.get("ShutdownIndependent").unwrap().lifecycle,
            LifecycleState::Unloaded
        );
    }
    let trace = recorded_calls(&calls);
    assert!(!trace.iter().any(|call| call == "ShutdownProvider:cleanup"));
    assert_eq!(
        trace
            .iter()
            .filter(|call| *call == "ShutdownIndependent:cleanup")
            .count(),
        1
    );

    let retry = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(retry.is_complete());
    assert_eq!(
        retry.completed,
        vec!["ShutdownDependent", "ShutdownProvider"]
    );
    let trace = recorded_calls(&calls);
    assert_eq!(
        trace
            .iter()
            .filter(|call| *call == "ShutdownDependent:cleanup")
            .count(),
        2
    );
    assert_eq!(
        trace
            .iter()
            .filter(|call| *call == "ShutdownProvider:cleanup")
            .count(),
        1
    );
    assert_eq!(
        trace
            .iter()
            .filter(|call| *call == "ShutdownIndependent:cleanup")
            .count(),
        1
    );
    assert!(handle.active_module_shutdown_order().is_empty());
    let repeated = runtime.shutdown_registered_modules_until(Instant::now());
    assert!(repeated.is_complete());
    assert!(repeated.attempted.is_empty());
}

#[test]
fn direct_provider_deactivation_rejects_a_stopping_dependent_without_services() {
    let (runtime, calls) = shutdown_fixture();
    assert!(runtime.deactivate_module("ShutdownDependent").is_err());
    assert!(matches!(runtime.deactivate_module("ShutdownProvider"),
        Err(CoreError::ModuleUnloadBlocked { dependents, .. })
        if dependents == vec!["ShutdownDependent"]));
    assert!(!recorded_calls(&calls)
        .iter()
        .any(|call| call == "ShutdownProvider:cleanup"));
    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::from_secs(1))
        .unwrap();
}

#[test]
fn expired_shutdown_report_names_every_unattempted_module_and_retains_retry() {
    let (runtime, calls) = shutdown_fixture();
    let handle = runtime.handle();
    let before = handle.active_module_shutdown_order();
    let report = runtime.shutdown_registered_modules_until(Instant::now());
    assert!(!report.is_complete());
    assert!(report.deadline_exhausted);
    assert!(report.attempted.is_empty());
    assert_eq!(
        report.not_attempted,
        before.iter().rev().cloned().collect::<Vec<_>>()
    );
    assert_eq!(handle.active_module_shutdown_order(), before);
    assert!(!recorded_calls(&calls)
        .iter()
        .any(|call| call.ends_with(":cleanup")));
    assert!(matches!(
        report.into_result(),
        Err(CoreError::ModuleCleanupTimeout {
            incomplete_entries: 3,
            ..
        })
    ));
    let first = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(!first.is_complete());
    let retry = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(retry.is_complete());
}

#[test]
fn zero_duration_shutdown_preserves_immediate_probe_and_independent_cleanup() {
    let (runtime, calls) = shutdown_fixture();
    assert!(matches!(
        runtime.shutdown_registered_modules_with_drain_timeout(Duration::ZERO),
        Err(CoreError::MissingConfig(_))
    ));
    assert_eq!(
        recorded_calls(&calls)
            .iter()
            .filter(|call| *call == "ShutdownIndependent:cleanup")
            .count(),
        1
    );
    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::ZERO)
        .unwrap();
    assert!(runtime.handle().active_module_shutdown_order().is_empty());
}

#[derive(Debug)]
struct NamedCleanupFailure;

impl ModuleLifecycle for NamedCleanupFailure {
    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        Err(CoreError::MissingConfig(context.module_name.clone()))
    }
}

#[test]
fn shutdown_result_preserves_first_error_after_later_independent_failure() {
    let runtime = CoreRuntime::new();
    for name in ["ShutdownLaterFailure", "ShutdownFirstFailure"] {
        runtime
            .register_module(
                ModuleDescriptor::new(name, "first traversal failure")
                    .with_lifecycle(Arc::new(NamedCleanupFailure)),
            )
            .unwrap();
    }
    // Activation freezes the registry graph; finish both registrations first.
    for name in ["ShutdownLaterFailure", "ShutdownFirstFailure"] {
        runtime.activate_module(name).unwrap();
    }
    let report = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert_eq!(report.failed.len(), 2);
    assert_eq!(report.failed[0].0, "ShutdownFirstFailure");
    assert_eq!(report.failed[1].0, "ShutdownLaterFailure");
    assert_eq!(
        report.into_result(),
        Err(CoreError::MissingConfig("ShutdownFirstFailure".to_owned()))
    );
}

#[derive(Debug)]
struct FinalUnloadDeadlineService {
    deadline: Arc<Mutex<Option<Instant>>>,
    drops: Arc<AtomicUsize>,
}

impl Drop for FinalUnloadDeadlineService {
    fn drop(&mut self) {
        let deadline = self.deadline.lock().unwrap().expect("cleanup entered");
        std::thread::sleep(
            deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
        );
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct RememberCleanupDeadline {
    deadline: Arc<Mutex<Option<Instant>>>,
    cleanups: Arc<AtomicUsize>,
}

impl ModuleLifecycle for RememberCleanupDeadline {
    fn cleanup_until(&self, _context: &ModuleContext, deadline: Instant) -> CoreResult<()> {
        *self.deadline.lock().unwrap() = Some(deadline);
        self.cleanups.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn final_service_unload_fixture() -> (CoreRuntime, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let runtime = CoreRuntime::new();
    let deadline = Arc::new(Mutex::new(None));
    let cleanups = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let factory_deadline = Arc::clone(&deadline);
    let factory_drops = Arc::clone(&drops);
    runtime
        .register_module(
            ModuleDescriptor::new(
                "ShutdownFinalUnload",
                "last service unload exceeds deadline",
            )
            .with_lifecycle(Arc::new(RememberCleanupDeadline {
                deadline,
                cleanups: Arc::clone(&cleanups),
            }))
            .with_manager(crate::core::ManagerDescriptor::new(
                crate::core::RegistryName::from_parts(
                    "ShutdownFinalUnload",
                    ServiceKind::Manager,
                    "FinalUnload",
                ),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(move |_context| {
                    Ok(Arc::new(FinalUnloadDeadlineService {
                        deadline: Arc::clone(&factory_deadline),
                        drops: Arc::clone(&factory_drops),
                    }) as ServiceObject)
                }),
            )),
        )
        .unwrap();
    runtime.activate_module("ShutdownFinalUnload").unwrap();
    (runtime, cleanups, drops)
}

#[test]
fn final_service_unload_overrun_is_completed_late_without_repeating_cleanup() {
    let (runtime, cleanups, drops) = final_service_unload_fixture();
    let report = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(cleanups.load(Ordering::SeqCst), 1);
    assert!(report.deadline_exhausted);
    assert!(!report.is_complete());
    assert_eq!(report.completed, ["ShutdownFinalUnload"]);
    assert_eq!(report.completed_after_deadline, ["ShutdownFinalUnload"]);
    assert!(report.failed.is_empty());
    assert!(report.not_attempted.is_empty());
    assert!(matches!(
        report.into_result(),
        Err(CoreError::ModuleCleanupTimeout { module, operation, incomplete_entries: 0, .. })
            if module == "ShutdownFinalUnload" && operation == "module_shutdown_completed_after_deadline"
    ));
    let retry = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(retry.is_complete());
    assert!(retry.attempted.is_empty());
    assert_eq!(cleanups.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn positive_duration_shutdown_reports_final_unload_lateness_without_repeating_cleanup() {
    let (runtime, cleanups, drops) = final_service_unload_fixture();
    let result = runtime.shutdown_registered_modules_with_drain_timeout(Duration::from_secs(1));
    assert!(matches!(
        result,
        Err(CoreError::ModuleCleanupTimeout { module, operation, incomplete_entries: 0, .. })
            if module == "ShutdownFinalUnload" && operation == "module_shutdown_completed_after_deadline"
    ));
    assert!(runtime.handle().active_module_shutdown_order().is_empty());
    assert_eq!(cleanups.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::from_secs(1))
        .unwrap();
    let retry = runtime.shutdown_registered_modules_until(Instant::now() + Duration::from_secs(1));
    assert!(retry.is_complete());
    assert!(retry.attempted.is_empty());
    assert!(retry.completed_after_deadline.is_empty());
    assert_eq!(cleanups.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
