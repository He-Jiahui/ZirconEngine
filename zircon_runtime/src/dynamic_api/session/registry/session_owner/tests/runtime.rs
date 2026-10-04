use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex};

use zircon_runtime_interface::{
    ZrRuntimePluginEventSubscribeRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::scene::RuntimeEventMirrorRegistration;

use super::*;

#[test]
fn failed_startup_callback_retains_core_scope_and_plugins_until_canonical_retry() {
    use crate::core::{
        CoreError, CoreRuntime, LifecycleState, TaskDescriptor, TaskGraphScopeDescriptor, TaskId,
        TaskPoolKind,
    };
    use crate::dynamic_api::session::construction::RuntimeConstructionCleanup;
    use crate::dynamic_api::session::linked_plugins::LinkedRuntimePluginPlan;
    use crate::dynamic_api::session::RuntimeDynamicSessionError;

    let runtime = CoreRuntime::try_new().expect("real Core for retained startup cleanup");
    let scope = runtime
        .create_task_graph_scope(TaskGraphScopeDescriptor::new("dynamic-session"))
        .expect("real dynamic session scope");
    let (modules, catalog_pin, plan_pin) = LinkedRuntimePluginPlan::prepare_core_only(
        RuntimeDynamicSessionProfile::Headless.target_mode(),
    )
    .expect("real headless module plan")
    .into_parts();
    for descriptor in modules.module_descriptors() {
        runtime
            .register_module(descriptor.clone())
            .expect("real module registration");
    }
    runtime
        .activate_registered_modules()
        .expect("real module activation");
    drop(modules);
    let core = runtime.handle();
    let catalog = Arc::downgrade(&catalog_pin);
    let plan = Arc::downgrade(&plan_pin);
    let mut checkpoint = RuntimeConstructionCleanup::new(&runtime, &catalog_pin, &plan_pin);
    checkpoint.attach_scope(&scope);
    drop(catalog_pin);
    drop(plan_pin);
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let _task = scope
        .submit(
            TaskDescriptor::new(TaskId::new(1), TaskPoolKind::Io, "retained-startup-work"),
            move |_| {
                started_tx.send(()).expect("notify task entry");
                let _ = release_rx.recv();
            },
        )
        .expect("real admitted scope task");
    started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("task entry barrier");
    let primary = RuntimeDynamicSessionError::CoreStep {
        step: "retention regression checkpoint",
        source: CoreError::Initialization("retention-test".to_owned(), "primary".to_owned()),
    };
    let failure = checkpoint
        .check_until::<()>(Err(primary), Instant::now())
        .expect_err("a blocked real scope must retain the unfinished Core");
    assert!(failure.has_pending_core());
    assert!(failure.shutdown_error().is_some());
    assert!(failure.shutdown_report().is_none());
    assert!(Arc::ptr_eq(
        &core.inner,
        &failure.observed_core.as_ref().unwrap().inner
    ));
    assert!(core
        .inner
        .modules
        .lock()
        .unwrap()
        .values()
        .any(|entry| entry.lifecycle == LifecycleState::Running));

    let log_calls = Arc::new(AtomicUsize::new(0));
    let log_calls_for_shutdown = Arc::clone(&log_calls);
    let startup = RuntimeSessionStartupFailure::capture(failure).with_dynamic_process_log(
        DynamicProcessLogLease::from_test_shutdown(move |_| {
            log_calls_for_shutdown.fetch_add(1, Ordering::SeqCst);
            true
        }),
    );
    let mut state: Result<RuntimeDynamicSession, RuntimeSessionStartupFailure> = Err(startup);
    assert!(!shutdown_owner_state_until(&mut state, Instant::now()));
    assert_eq!(log_calls.load(Ordering::SeqCst), 0);
    assert!(catalog.upgrade().is_some());
    assert!(plan.upgrade().is_some());
    release_tx.send(()).expect("release the real scope task");
    assert!(shutdown_owner_state_until(
        &mut state,
        Instant::now() + Duration::from_secs(5)
    ));
    let startup = state.as_ref().err().unwrap();
    assert!(!startup.construction.has_pending_core());
    let report = startup
        .construction
        .shutdown_report()
        .expect("real Core shutdown report");
    assert!(!report.has_in_flight_work());
    assert_eq!(report.worker_shutdowns.len(), 3);
    assert!(report
        .worker_shutdowns
        .iter()
        .all(|workers| workers.all_joined()));
    assert!(core
        .inner
        .modules
        .lock()
        .unwrap()
        .values()
        .all(|entry| entry.lifecycle == LifecycleState::Unloaded));
    assert_eq!(log_calls.load(Ordering::SeqCst), 1);
    drop(checkpoint);
    drop(state);
    assert!(catalog.upgrade().is_none());
    assert!(plan.upgrade().is_none());
}

#[test]
fn astra_life_a4_runtime_owner_bounds_blocking_event_callback_without_reinvocation() {
    const EVENT: &str = "astra.owner.event";
    const SCHEMA: &str = "astra.owner.event.v1";
    const SHUTDOWN_BUDGET: Duration = Duration::from_millis(25);
    // Allow 10x scheduler and lock-handoff jitter on Windows while still detecting an unbounded wait.
    const SHUTDOWN_SCHEDULER_CEILING: Duration = Duration::from_millis(250);
    let code_owner = Arc::new(());
    let weak_code_owner = Arc::downgrade(&code_owner);
    let owner = RuntimeSessionOwner::create(
        RuntimeDynamicSessionProfile::Headless,
        None,
        RuntimeWakeRegistration::disabled(),
        code_owner,
    )
    .unwrap();
    let invocations = Arc::new(AtomicUsize::new(0));
    let callback_invocations = Arc::clone(&invocations);
    let released = Arc::new(AtomicBool::new(false));
    let released_for_callback = Arc::clone(&released);
    let callback_completed = Arc::new(AtomicBool::new(false));
    let callback_completed_for_callback = Arc::clone(&callback_completed);
    let (callback_started_tx, callback_started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let release_rx = Mutex::new(release_rx);
    owner
        .dispatch(Instant::now() + Duration::from_secs(5), move |session| {
            let owner_thread = std::thread::current().id();
            session.level.with_world_mut(|world| {
                world
                    .register_runtime_event_mirror(
                        RuntimeEventMirrorRegistration::typed::<u32>(EVENT, SCHEMA)
                            .with_reader_count_callback(move |_world, count| {
                                assert_eq!(owner_thread, std::thread::current().id());
                                if count == 0 {
                                    callback_invocations.fetch_add(1, Ordering::SeqCst);
                                    if !released_for_callback.load(Ordering::Acquire) {
                                        callback_started_tx.send(()).unwrap();
                                        release_rx.lock().unwrap().recv().unwrap();
                                    }
                                    callback_completed_for_callback.store(true, Ordering::Release);
                                }
                                Ok(())
                            }),
                    )
                    .unwrap();
            });
            session
                .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    EVENT,
                    SCHEMA,
                ))
                .unwrap();
            Ok(())
        })
        .unwrap()
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
        .unwrap();

    let first_shutdown_started_at = Instant::now();
    assert_eq!(
        owner.shutdown_until(Instant::now() + SHUTDOWN_BUDGET),
        OwnerShutdownReceipt::Pending
    );
    assert!(first_shutdown_started_at.elapsed() <= SHUTDOWN_SCHEDULER_CEILING);
    callback_started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("shutdown must start the event callback before returning Pending");
    assert!(!released.load(Ordering::Acquire));
    assert!(!callback_completed.load(Ordering::Acquire));
    let retry_shutdown_started_at = Instant::now();
    assert_eq!(
        owner.shutdown_until(Instant::now() + SHUTDOWN_BUDGET),
        OwnerShutdownReceipt::Pending
    );
    assert!(retry_shutdown_started_at.elapsed() <= SHUTDOWN_SCHEDULER_CEILING);
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert!(!released.load(Ordering::Acquire));
    assert!(!callback_completed.load(Ordering::Acquire));
    assert!(weak_code_owner.upgrade().is_some());
    released.store(true, Ordering::Release);
    release_tx.send(()).unwrap();
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(5)),
        OwnerShutdownReceipt::Retryable
    );
    assert!(callback_completed.load(Ordering::Acquire));
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(5)),
        OwnerShutdownReceipt::Joined
    );
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert!(weak_code_owner.upgrade().is_some());
    drop(owner);
    assert!(weak_code_owner.upgrade().is_none());
}
