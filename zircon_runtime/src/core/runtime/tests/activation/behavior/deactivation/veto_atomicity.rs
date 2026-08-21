use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{panic, panic::AssertUnwindSafe};

use super::super::super::super::super::*;
use crate::core::{
    CoreError, CoreResult, LifecycleState, ModuleContext, ModuleLifecycle,
    RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver,
};

#[derive(Debug)]
struct CleanupCounterLifecycle {
    cleanup_count: Arc<AtomicUsize>,
}

impl ModuleLifecycle for CleanupCounterLifecycle {
    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.cleanup_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Debug)]
struct BuildCounterLifecycle {
    build_count: Arc<AtomicUsize>,
}

impl ModuleLifecycle for BuildCounterLifecycle {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.build_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Debug)]
struct PanicCleanupLifecycle;

impl ModuleLifecycle for PanicCleanupLifecycle {
    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        panic!("m2 module cleanup panic")
    }
}

#[derive(Debug)]
struct DeactivationVeto;

impl RuntimeModuleLifecycleObserver for DeactivationVeto {
    fn runtime_module_activated(&self, _module_name: &str) {}

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Err(RuntimeModuleLifecycleBlock::new("m0 deactivation veto"))
    }
}

#[test]
fn deactivation_veto_prepares_without_cleanup_and_preserves_running_state() {
    let runtime = CoreRuntime::new();
    let cleanup_count = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("VetoAtomicityModule", "M0 veto atomicity").with_lifecycle(
                Arc::new(CleanupCounterLifecycle {
                    cleanup_count: Arc::clone(&cleanup_count),
                }),
            ),
        )
        .unwrap();
    runtime.activate_module("VetoAtomicityModule").unwrap();
    runtime.install_runtime_module_lifecycle_observer(Arc::new(DeactivationVeto));

    let error = runtime
        .deactivate_module("VetoAtomicityModule")
        .unwrap_err();

    assert!(matches!(
        error,
        CoreError::RuntimeModuleLifecycleBlocked(detail) if detail == "m0 deactivation veto"
    ));
    assert_eq!(
        cleanup_count.load(Ordering::SeqCst),
        0,
        "a prepare-phase veto must run before cleanup side effects"
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("VetoAtomicityModule")
        .expect("vetoed module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
}

#[test]
fn deactivation_rejects_a_registered_module_without_cleanup() {
    let runtime = CoreRuntime::new();
    let cleanup_count = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("RegisteredStopModule", "registered stop guard").with_lifecycle(
                Arc::new(CleanupCounterLifecycle {
                    cleanup_count: Arc::clone(&cleanup_count),
                }),
            ),
        )
        .unwrap();

    let error = runtime
        .deactivate_module("RegisteredStopModule")
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::InvalidModuleLifecycleTransition {
            module,
            command,
            state: LifecycleState::Registered,
        } if module == "RegisteredStopModule" && command == "deactivate"
    ));
    assert_eq!(
        cleanup_count.load(Ordering::SeqCst),
        0,
        "deactivation cannot clean up a module that was never activated"
    );
}

#[test]
fn deactivation_rejects_running_module_dependents_before_cleanup() {
    let runtime = CoreRuntime::new();
    let cleanup_count = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("ModuleUnloadProvider", "dependency provider").with_lifecycle(
                Arc::new(CleanupCounterLifecycle {
                    cleanup_count: Arc::clone(&cleanup_count),
                }),
            ),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ModuleUnloadDependent", "dependency consumer")
                .with_module_dependency(ModuleDependencySpec::named("ModuleUnloadProvider")),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ModuleUnloadTransitiveDependent", "terminal consumer")
                .with_module_dependency(ModuleDependencySpec::named("ModuleUnloadDependent")),
        )
        .unwrap();
    runtime
        .activate_module("ModuleUnloadTransitiveDependent")
        .unwrap();

    let error = runtime
        .deactivate_module("ModuleUnloadProvider")
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ModuleUnloadBlocked { module, dependents }
            if module == "ModuleUnloadProvider"
                && dependents
                    == vec![
                        "ModuleUnloadDependent".to_owned(),
                        "ModuleUnloadTransitiveDependent".to_owned(),
                    ]
    ));
    assert_eq!(
        cleanup_count.load(Ordering::SeqCst),
        0,
        "module dependent validation must veto before provider cleanup"
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("ModuleUnloadProvider")
            .expect("provider should remain registered")
            .lifecycle,
        LifecycleState::Running
    );
}

#[test]
fn runtime_shutdown_unloads_modules_in_reverse_dependency_order() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new(
            "RuntimeShutdownProvider",
            "shutdown provider",
        ))
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("RuntimeShutdownConsumer", "shutdown consumer")
                .with_module_dependency(ModuleDependencySpec::named("RuntimeShutdownProvider")),
        )
        .unwrap();
    runtime.activate_module("RuntimeShutdownConsumer").unwrap();

    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::ZERO)
        .expect("reverse graph shutdown must unload the consumer before its provider");

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module_name in ["RuntimeShutdownProvider", "RuntimeShutdownConsumer"] {
        assert_eq!(
            modules
                .get(module_name)
                .expect("registered shutdown module")
                .lifecycle,
            LifecycleState::Unloaded
        );
    }
}

#[test]
fn runtime_shutdown_skips_registered_modules_and_cleans_running_modules() {
    let runtime = CoreRuntime::new();
    let cleanup_count = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("RuntimeShutdownActive", "running shutdown module")
                .with_lifecycle(Arc::new(CleanupCounterLifecycle {
                    cleanup_count: Arc::clone(&cleanup_count),
                })),
        )
        .unwrap();
    runtime
        .register_module(ModuleDescriptor::new(
            "RuntimeShutdownDormant",
            "registered shutdown module",
        ))
        .unwrap();
    runtime.activate_module("RuntimeShutdownActive").unwrap();
    let handle = runtime.handle();
    let expected_activation_order = [
        "RuntimeShutdownActive".to_owned(),
        "RuntimeShutdownDormant".to_owned(),
    ];
    assert_eq!(
        handle
            .frozen_module_graph()
            .unwrap()
            .module_activation_order(),
        expected_activation_order.as_slice()
    );

    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::ZERO)
        .expect("a never-started module must not block cleanup of a running module");

    assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("RuntimeShutdownActive")
            .expect("running shutdown module")
            .lifecycle,
        LifecycleState::Unloaded
    );
    assert_eq!(
        modules
            .get("RuntimeShutdownDormant")
            .expect("registered shutdown module")
            .lifecycle,
        LifecycleState::Registered
    );
}

#[test]
fn runtime_shutdown_active_module_order_tracks_successful_reactivation() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new(
            "RuntimeShutdownLedgerProvider",
            "shutdown ledger provider",
        ))
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("RuntimeShutdownLedgerConsumer", "shutdown ledger consumer")
                .with_module_dependency(ModuleDependencySpec::named(
                    "RuntimeShutdownLedgerProvider",
                )),
        )
        .unwrap();

    runtime
        .activate_module("RuntimeShutdownLedgerConsumer")
        .unwrap();
    assert_eq!(
        runtime.handle().active_module_shutdown_order(),
        [
            "RuntimeShutdownLedgerProvider".to_owned(),
            "RuntimeShutdownLedgerConsumer".to_owned(),
        ]
    );

    runtime
        .deactivate_module("RuntimeShutdownLedgerConsumer")
        .unwrap();
    assert_eq!(
        runtime.handle().active_module_shutdown_order(),
        ["RuntimeShutdownLedgerProvider".to_owned()]
    );

    runtime
        .activate_module("RuntimeShutdownLedgerConsumer")
        .unwrap();
    assert_eq!(
        runtime.handle().active_module_shutdown_order(),
        [
            "RuntimeShutdownLedgerProvider".to_owned(),
            "RuntimeShutdownLedgerConsumer".to_owned(),
        ]
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime_shutdown_active_ledger_release_benchmark_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    const DECLARED_MODULES: usize = 16_384;
    const ACTIVE_MODULES: usize = 8;
    const SAMPLE_PAIRS: usize = 21;
    const REGISTRY_VISIT_REDUCTION_BASIS_POINTS: usize =
        (DECLARED_MODULES - ACTIVE_MODULES) * 10_000 / DECLARED_MODULES;

    let runtime = CoreRuntime::new();
    let module_names = (0..DECLARED_MODULES)
        .map(|index| format!("RuntimeShutdownLedger{index:05}"))
        .collect::<Vec<_>>();
    for module_name in &module_names {
        runtime
            .register_module(ModuleDescriptor::new(
                module_name.clone(),
                "shutdown ledger release benchmark",
            ))
            .unwrap();
    }
    for module_name in module_names.iter().take(ACTIVE_MODULES) {
        runtime.activate_module(module_name).unwrap();
    }

    let handle = runtime.handle();
    let graph = handle.frozen_module_graph().unwrap();
    let expected = module_names[..ACTIVE_MODULES].to_vec();
    assert_eq!(graph.module_activation_order().len(), DECLARED_MODULES);
    assert_eq!(handle.active_module_shutdown_order(), expected);

    let legacy_shutdown_order = || {
        let modules = handle.lock_modules();
        graph
            .module_activation_order()
            .iter()
            .filter(|module_name| {
                modules
                    .get(*module_name)
                    .is_some_and(|entry| entry.lifecycle == LifecycleState::Running)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let started = Instant::now();
            assert_eq!(black_box(legacy_shutdown_order()), expected);
            legacy_ns.push(started.elapsed().as_nanos());

            let started = Instant::now();
            assert_eq!(black_box(handle.active_module_shutdown_order()), expected);
            optimized_ns.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            assert_eq!(black_box(handle.active_module_shutdown_order()), expected);
            optimized_ns.push(started.elapsed().as_nanos());

            let started = Instant::now();
            assert_eq!(black_box(legacy_shutdown_order()), expected);
            legacy_ns.push(started.elapsed().as_nanos());
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "active-ledger P95 must be at least 75% below the full registry scan: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME72_SHUTDOWN_ACTIVE_LEDGER_BENCH_V1 declared_modules={DECLARED_MODULES} active_modules={ACTIVE_MODULES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_registry_visits={DECLARED_MODULES} optimized_ledger_copies={ACTIVE_MODULES} registry_visit_reduction_basis_points={REGISTRY_VISIT_REDUCTION_BASIS_POINTS} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn deactivation_callback_panic_is_reported_as_a_typed_failure() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(
            ModuleDescriptor::new("PanicCleanupModule", "M2 callback panic")
                .with_lifecycle(Arc::new(PanicCleanupLifecycle)),
        )
        .unwrap();
    runtime.activate_module("PanicCleanupModule").unwrap();

    let deactivation = panic::catch_unwind(AssertUnwindSafe(|| {
        runtime.deactivate_module("PanicCleanupModule")
    }));
    let deactivation =
        deactivation.expect("a module cleanup panic must not escape the lifecycle API");
    assert!(matches!(
        deactivation,
        Err(CoreError::ModuleLifecycleCallbackPanicked { module, command })
            if module == "PanicCleanupModule" && command == "deactivate"
    ));

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("PanicCleanupModule")
        .expect("failed cleanup must retain its module slot");
    assert_eq!(
        module.lifecycle,
        LifecycleState::Stopping,
        "a failure after deactivation commit must not restore a running module"
    );
    drop(modules);

    let error = runtime.activate_module("PanicCleanupModule").unwrap_err();
    assert!(matches!(
        error,
        CoreError::InvalidModuleLifecycleTransition {
            module,
            command,
            state: LifecycleState::Stopping,
        } if module == "PanicCleanupModule" && command == "activate"
    ));
}

#[test]
fn deactivation_drain_deadline_keeps_the_committed_module_stopping() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "DrainDeadlineModule",
        ServiceKind::Manager,
        "DrainDeadlineManager",
    );
    runtime
        .register_module(
            ModuleDescriptor::new("DrainDeadlineModule", "M3 drain deadline").with_manager(
                ManagerDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    runtime.activate_module("DrainDeadlineModule").unwrap();

    let service = runtime
        .resolve_manager_handle::<()>(service_name.as_str())
        .unwrap();
    let held_call = service.enter().unwrap();

    let error = runtime
        .deactivate_module_with_drain_timeout("DrainDeadlineModule", Duration::ZERO)
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ServiceCallDrainTimeout {
            module,
            budget,
            in_flight_calls: 1,
        } if module == "DrainDeadlineModule" && budget == Duration::ZERO
    ));

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("DrainDeadlineModule")
            .expect("the timed-out module should retain its slot")
            .lifecycle,
        LifecycleState::Stopping,
        "a committed drain failure must not restore a callable module"
    );
    drop(modules);

    assert!(matches!(
        service.enter(),
        Err(CoreError::ServiceUnavailable(name)) if name == service_name.as_str()
    ));
    drop(held_call);

    runtime
        .deactivate_module_with_drain_timeout("DrainDeadlineModule", Duration::ZERO)
        .expect("a committed drain timeout should resume once the final call guard releases");
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("DrainDeadlineModule")
            .expect("the resumed module should retain its slot")
            .lifecycle,
        LifecycleState::Unloaded
    );
}

#[test]
fn activation_preflight_rejects_stopping_dependents_before_reactivating_dependencies() {
    let runtime = CoreRuntime::new();
    let dependency_builds = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("PreflightDependency", "closure dependency").with_lifecycle(
                Arc::new(BuildCounterLifecycle {
                    build_count: Arc::clone(&dependency_builds),
                }),
            ),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("PreflightStoppingTarget", "stopping target")
                .with_module_dependency(ModuleDependencySpec::named("PreflightDependency"))
                .with_lifecycle(Arc::new(PanicCleanupLifecycle)),
        )
        .unwrap();
    runtime.activate_module("PreflightStoppingTarget").unwrap();
    assert_eq!(dependency_builds.load(Ordering::SeqCst), 1);

    let deactivation = runtime.deactivate_module("PreflightStoppingTarget");
    assert!(matches!(
        deactivation,
        Err(CoreError::ModuleLifecycleCallbackPanicked { module, command })
            if module == "PreflightStoppingTarget" && command == "deactivate"
    ));
    runtime.deactivate_module("PreflightDependency").unwrap();

    let error = runtime
        .activate_module("PreflightStoppingTarget")
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::InvalidModuleLifecycleTransition {
            module,
            command,
            state: LifecycleState::Stopping,
        } if module == "PreflightStoppingTarget" && command == "activate"
    ));
    assert_eq!(
        dependency_builds.load(Ordering::SeqCst),
        1,
        "a stopping target must reject its closure before an unloaded dependency rebuilds"
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    assert_eq!(
        modules
            .get("PreflightDependency")
            .expect("dependency should remain registered")
            .lifecycle,
        LifecycleState::Unloaded
    );
}
