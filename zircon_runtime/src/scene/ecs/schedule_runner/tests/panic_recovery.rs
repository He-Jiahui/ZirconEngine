use super::*;

#[test]
fn panicking_runtime_system_is_restored_before_the_panic_resumes() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let panic_once_for_system = Arc::clone(&panic_once);
    let calls_for_system = Arc::clone(&calls);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "tests.runtime_panic_once",
            SystemStage::Update,
            move || {
                let panic_once = Arc::clone(&panic_once_for_system);
                let calls = Arc::clone(&calls_for_system);
                move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    if panic_once.swap(false, Ordering::SeqCst) {
                        panic!("intentional runtime callback panic");
                    }
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    let first =
        std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
    assert!(first.is_err());
    run_test_stage(&core.handle(), &level);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn panicking_worker_safe_system_is_restored_before_the_panic_resumes() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let panic_once_for_system = panic_once.clone();
    let calls_for_system = calls.clone();
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_system(
            owner,
            "tests.panic_once",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |world| {
                let resource_id = world.external_resource_id("tests.panic_resource");
                let mut access = SystemParamAccess::default();
                access
                    .add_resource_write(resource_id)
                    .map_err(|error| error.to_string())?;
                Ok(access)
            },
            move || {
                let calls = Arc::clone(&calls_for_system);
                let panic_once = Arc::clone(&panic_once_for_system);
                move || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    if panic_once.swap(false, Ordering::SeqCst) {
                        panic!("intentional worker callback panic");
                    }
                }
            },
        )
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    let first =
        std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
    assert!(first.is_err());
    run_test_stage(&core.handle(), &level);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn panicking_worker_command_restores_native_system_before_the_panic_resumes() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let panic_once_for_system = panic_once.clone();
    let calls_for_system = calls.clone();
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.command_panic_once",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let calls = Arc::clone(&calls_for_system);
                let panic_once = Arc::clone(&panic_once_for_system);
                move |commands| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    let panic_once_for_command = Arc::clone(&panic_once);
                    commands.push(move |_world: &mut World| {
                        if panic_once_for_command.swap(false, Ordering::SeqCst) {
                            panic!("intentional worker command panic");
                        }
                    });
                }
            },
        )
        .with_command_capacity(1)
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    let first =
        std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
    assert!(first.is_err());
    run_test_stage(&core.handle(), &level);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn panicking_worker_callback_discards_its_local_commands_before_retry() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let published = Arc::new(AtomicUsize::new(0));
    let panic_once_for_system = Arc::clone(&panic_once);
    let calls_for_system = Arc::clone(&calls);
    let published_for_system = Arc::clone(&published);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.callback_panic_discards_local_commands",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let panic_once = Arc::clone(&panic_once_for_system);
                let calls = Arc::clone(&calls_for_system);
                let published = Arc::clone(&published_for_system);
                move |commands| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    let published = Arc::clone(&published);
                    commands.push(move |_world: &mut World| {
                        published.fetch_add(1, Ordering::SeqCst);
                    });
                    if panic_once.swap(false, Ordering::SeqCst) {
                        panic!("intentional worker callback panic after enqueue");
                    }
                }
            },
        )
        .with_command_capacity(1)
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    let first =
        std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
    assert!(first.is_err());
    assert_eq!(published.load(Ordering::SeqCst), 0);

    run_test_stage(&core.handle(), &level);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(published.load(Ordering::SeqCst), 1);
}

#[test]
fn panicking_worker_callback_discards_sibling_worker_commands_before_retry() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let first_window = Arc::new(AtomicBool::new(true));
    let first_window_barrier = Arc::new(Barrier::new(2));
    let published = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();

    let first_window_for_sibling = Arc::clone(&first_window);
    let first_window_barrier_for_sibling = Arc::clone(&first_window_barrier);
    let published_for_sibling = Arc::clone(&published);
    registry
        .register_external_native_command_system(
            owner,
            "tests.sibling_worker_command",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let first_window = Arc::clone(&first_window_for_sibling);
                let first_window_barrier = Arc::clone(&first_window_barrier_for_sibling);
                let published = Arc::clone(&published_for_sibling);
                move |commands| {
                    let published = Arc::clone(&published);
                    commands.push(move |_world: &mut World| {
                        published.fetch_add(1, Ordering::SeqCst);
                    });
                    if first_window.load(Ordering::SeqCst) {
                        first_window_barrier.wait();
                    }
                }
            },
        )
        .with_order(10)
        .with_command_capacity(1)
        .register()
        .unwrap();

    let panic_once_for_system = Arc::clone(&panic_once);
    let first_window_for_panic = Arc::clone(&first_window);
    let first_window_barrier_for_panic = Arc::clone(&first_window_barrier);
    registry
        .register_external_native_command_system(
            owner,
            "tests.panic_after_sibling_enqueue",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let panic_once = Arc::clone(&panic_once_for_system);
                let first_window = Arc::clone(&first_window_for_panic);
                let first_window_barrier = Arc::clone(&first_window_barrier_for_panic);
                move |_commands| {
                    if panic_once.swap(false, Ordering::SeqCst) {
                        first_window_barrier.wait();
                        first_window.store(false, Ordering::SeqCst);
                        panic!("intentional worker callback panic after sibling enqueue");
                    }
                }
            },
        )
        .with_order(20)
        .register()
        .unwrap();

    let level = test_level(registry);
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ));
    let mut dispatches = vec![
        WorkerDispatch {
            id: "tests.sibling_worker_command",
            key: DeferredSystemKey::compiled(
                SystemStage::Update.rank(),
                10,
                "tests.sibling_worker_command",
            ),
        },
        WorkerDispatch {
            id: "tests.panic_after_sibling_enqueue",
            key: DeferredSystemKey::compiled(
                SystemStage::Update.rank(),
                20,
                "tests.panic_after_sibling_enqueue",
            ),
        },
    ];

    let first = std::panic::catch_unwind(AssertUnwindSafe(|| {
        flush_worker_batch(&scheduler, &level, &mut dispatches, true)
    }));
    assert!(first.is_err());
    assert_eq!(published.load(Ordering::SeqCst), 0);

    dispatches = vec![
        WorkerDispatch {
            id: "tests.sibling_worker_command",
            key: DeferredSystemKey::compiled(
                SystemStage::Update.rank(),
                10,
                "tests.sibling_worker_command",
            ),
        },
        WorkerDispatch {
            id: "tests.panic_after_sibling_enqueue",
            key: DeferredSystemKey::compiled(
                SystemStage::Update.rank(),
                20,
                "tests.panic_after_sibling_enqueue",
            ),
        },
    ];
    flush_worker_batch(&scheduler, &level, &mut dispatches, true)
        .expect("clean retry must reuse the restored worker systems");

    assert_eq!(published.load(Ordering::SeqCst), 1);
}
