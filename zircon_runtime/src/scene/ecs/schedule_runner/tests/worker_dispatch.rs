use super::*;

#[test]
fn disjoint_worker_safe_native_systems_overlap_in_the_production_stage_runner() {
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    register_timed_external_system(
        &mut registry,
        owner,
        "tests.worker_a",
        "tests.resource_a",
        active.clone(),
        max_active.clone(),
    );
    register_timed_external_system(
        &mut registry,
        owner,
        "tests.worker_b",
        "tests.resource_b",
        active,
        max_active.clone(),
    );
    let core = CoreRuntime::new();
    let level = test_level(registry);

    run_test_stage(&core.handle(), &level);

    let expected = usize::from(core.scheduler().parallelism() > 1) + 1;
    assert_eq!(max_active.load(Ordering::SeqCst), expected);
    let diagnostics = level.with_world(|world| {
        world
            .ecs_frame_performance_diagnostics()
            .native_system_schedule
    });
    assert_eq!(diagnostics.conflict_count(), 0);
    assert_eq!(diagnostics.worker_batch_count(), 1);
    assert_eq!(diagnostics.callback_count(), 2);
    assert_eq!(diagnostics.temporary_control_buffer_count(), 2);
    assert!(diagnostics.temporary_control_buffer_bytes() > 0);
    assert!(diagnostics.callback_p95_ms() >= 30.0);
    assert!(diagnostics.worker_utilization() > 0.0);
}

#[test]
fn conflicting_worker_safe_native_systems_remain_serial() {
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    register_timed_external_system(
        &mut registry,
        owner,
        "tests.conflict_a",
        "tests.shared_resource",
        active.clone(),
        max_active.clone(),
    );
    register_timed_external_system(
        &mut registry,
        owner,
        "tests.conflict_b",
        "tests.shared_resource",
        active,
        max_active.clone(),
    );
    let core = CoreRuntime::new();
    let level = test_level(registry);

    run_test_stage(&core.handle(), &level);

    assert_eq!(max_active.load(Ordering::SeqCst), 1);
    let diagnostics = level.with_world(|world| {
        world
            .ecs_frame_performance_diagnostics()
            .native_system_schedule
    });
    assert_eq!(diagnostics.conflict_count(), 1);
    assert_eq!(diagnostics.worker_batch_count(), 2);
    assert_eq!(diagnostics.callback_count(), 2);
}

#[test]
fn main_thread_only_external_system_runs_on_the_schedule_caller() {
    let caller = thread::current().id();
    let observed = Arc::new(Mutex::new(None));
    let observed_for_system = observed.clone();
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_system(
            owner,
            "tests.main_thread",
            SystemStage::Update,
            SceneSystemThreadAffinity::MainThreadOnly,
            |_world| {
                let mut access = SystemParamAccess::default();
                access.add_conservative_world_access();
                Ok(access)
            },
            move || {
                let observed = Arc::clone(&observed_for_system);
                move || {
                    *observed.lock().unwrap() = Some(thread::current().id());
                }
            },
        )
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    run_test_stage(&core.handle(), &level);

    assert_eq!(*observed.lock().unwrap(), Some(caller));
    let diagnostics = level.with_world(|world| {
        world
            .ecs_frame_performance_diagnostics()
            .native_system_schedule
    });
    assert_eq!(diagnostics.worker_batch_count(), 0);
    assert_eq!(diagnostics.callback_count(), 1);
    assert_eq!(diagnostics.conservative_world_writer_count(), 1);
}

#[test]
fn conservative_world_writer_is_not_dispatched_to_a_worker() {
    let caller = thread::current().id();
    let observed = Arc::new(Mutex::new(None));
    let observed_for_system = observed.clone();
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_system(
            owner,
            "tests.conservative_world_writer",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| {
                let mut access = SystemParamAccess::default();
                access.add_conservative_world_access();
                Ok(access)
            },
            move || {
                let observed = Arc::clone(&observed_for_system);
                move || {
                    *observed.lock().unwrap() = Some(thread::current().id());
                }
            },
        )
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    run_test_stage(&core.handle(), &level);

    assert_eq!(*observed.lock().unwrap(), Some(caller));
    let diagnostics = level.with_world(|world| {
        world
            .ecs_frame_performance_diagnostics()
            .native_system_schedule
    });
    assert_eq!(diagnostics.worker_batch_count(), 0);
    assert_eq!(diagnostics.callback_count(), 1);
    assert_eq!(diagnostics.conservative_world_writer_count(), 1);
}
