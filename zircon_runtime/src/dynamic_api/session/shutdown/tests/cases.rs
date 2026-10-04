use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use crate::core::{
    CoreError, CoreResult, CoreRuntime, CoreShutdownError, ModuleContext, ModuleDescriptor,
    ModuleLifecycle, TaskCancellationPolicy, TaskDescriptor, TaskGraphScope,
    TaskGraphScopeDescriptor, TaskGraphShutdownReport, TaskId, TaskPoolKind,
};

use super::shutdown_runtime_core_until;

const WAIT_BUDGET: Duration = Duration::from_secs(2);

struct ObservedCleanup {
    calls: Arc<AtomicUsize>,
    task_finished: Arc<AtomicBool>,
    scope: Option<TaskGraphScope>,
    fail_once: bool,
}

impl ModuleLifecycle for ObservedCleanup {
    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        let attempt = self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(self.task_finished.load(Ordering::Acquire));
        if let Some(scope) = &self.scope {
            let census = scope.census();
            assert!(!census.accepting);
            assert!(census.is_quiescent());
        }
        if attempt == 0 {
            // A real graph admission proves graph shutdown has not preceded modules.
            let core = context.core.upgrade().expect("cleanup retains its Core");
            let probe = core
                .task_graph()
                .create_scope(TaskGraphScopeDescriptor::new("cleanup-order-probe"))
                .expect("graph admission must remain open until module cleanup");
            drop(probe);
        }
        if self.fail_once && attempt == 0 {
            return Err(CoreError::MissingConfig("cleanup-order-fixture".to_owned()));
        }
        Ok(())
    }
}

fn install_cleanup(runtime: &CoreRuntime, cleanup: ObservedCleanup) {
    runtime
        .register_module(
            ModuleDescriptor::new("ObservedShutdown", "dynamic close ordering")
                .with_lifecycle(Arc::new(cleanup)),
        )
        .unwrap();
    runtime.activate_registered_modules().unwrap();
}

fn assert_graph_joined(report: &TaskGraphShutdownReport) {
    assert!(!report.has_in_flight_work());
    assert!(report.timer_joined);
    for kind in [
        TaskPoolKind::Compute,
        TaskPoolKind::AsyncCompute,
        TaskPoolKind::Io,
    ] {
        assert!(report.worker_shutdown(kind).unwrap().all_joined());
    }
}

#[test]
fn dynamic_core_close_drains_scope_before_modules_and_joins_graph_after_cleanup() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let scope = core
        .task_graph()
        .create_scope(TaskGraphScopeDescriptor::new("dynamic-close-scope"))
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let task_finished = Arc::new(AtomicBool::new(false));
    install_cleanup(
        &runtime,
        ObservedCleanup {
            calls: Arc::clone(&calls),
            task_finished: Arc::clone(&task_finished),
            scope: Some(scope.clone()),
            fail_once: false,
        },
    );
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel::<()>(1);
    let finished = Arc::clone(&task_finished);
    let task = scope
        .submit(
            TaskDescriptor::new(TaskId::new(1), TaskPoolKind::Compute, "close-order")
                .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                let _ = started_tx.send(());
                // A panic in the parent also drops the sole release sender.
                let _ = release_rx.recv();
                finished.store(true, Ordering::Release);
            },
        )
        .unwrap();
    started_rx.recv_timeout(WAIT_BUDGET).unwrap();

    let mut watchers_shutdown = false;
    let pending = shutdown_runtime_core_until(
        &runtime,
        Some(&scope),
        &mut watchers_shutdown,
        Instant::now(),
    )
    .unwrap_err();
    assert!(pending.to_string().contains("dynamic session scope"));
    assert!(pending.source().is_none());
    assert!(watchers_shutdown);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(!scope.census().accepting);
    assert!(!scope.census().is_quiescent());

    drop(release_tx);
    let report = shutdown_runtime_core_until(
        &runtime,
        Some(&scope),
        &mut watchers_shutdown,
        Instant::now() + WAIT_BUDGET,
    )
    .unwrap();
    assert_graph_joined(&report);
    assert!(task.is_complete());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(core.active_module_shutdown_order().is_empty());
    assert!(core
        .task_graph()
        .create_scope(TaskGraphScopeDescriptor::new("after-close"))
        .is_err());
}

#[test]
fn dynamic_core_error_preserves_combined_results_and_exact_retry() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let calls = Arc::new(AtomicUsize::new(0));
    install_cleanup(
        &runtime,
        ObservedCleanup {
            calls: Arc::clone(&calls),
            task_finished: Arc::new(AtomicBool::new(true)),
            scope: None,
            fail_once: true,
        },
    );
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel::<()>(1);
    core.task_graph().worker_pool().spawn(move || {
        let _ = started_tx.send(());
        let _ = release_rx.recv();
    });
    started_rx.recv_timeout(WAIT_BUDGET).unwrap();

    let mut watchers_shutdown = false;
    let first = shutdown_runtime_core_until(
        &runtime,
        None,
        &mut watchers_shutdown,
        Instant::now() + WAIT_BUDGET,
    )
    .unwrap_err();
    let aggregate = first
        .source()
        .unwrap()
        .downcast_ref::<CoreShutdownError>()
        .expect("dynamic close must retain the full Core aggregate");
    assert_eq!(
        aggregate.module_error(),
        Some(&CoreError::MissingConfig(
            "cleanup-order-fixture".to_owned()
        ))
    );
    let graph_error = aggregate.task_graph_error().unwrap();
    assert_eq!(aggregate.graph_report(), Some(&graph_error.report));
    assert!(graph_error.report.has_in_flight_work());
    let diagnostic = first.to_string();
    assert!(diagnostic.contains("registered module shutdown failed"));
    assert!(diagnostic.contains("owned task graph shutdown failed"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        core.active_module_shutdown_order(),
        vec!["ObservedShutdown"]
    );

    drop(release_tx);
    let report = shutdown_runtime_core_until(
        &runtime,
        None,
        &mut watchers_shutdown,
        Instant::now() + WAIT_BUDGET,
    )
    .unwrap();
    assert_graph_joined(&report);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(core.active_module_shutdown_order().is_empty());
    assert!(aggregate.task_graph_error().is_some());
    assert!(aggregate.module_error().is_some());
}
