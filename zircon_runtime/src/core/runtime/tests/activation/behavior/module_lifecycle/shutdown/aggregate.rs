use std::error::Error;
use std::sync::mpsc::{self, SyncSender};

use super::*;
use crate::core::{
    CoreShutdownError, TaskGraphShutdownError, TaskGraphShutdownReport, TaskPoolKind,
};

const SHUTDOWN_BUDGET: Duration = Duration::from_secs(2);
const WORKER_START_BUDGET: Duration = Duration::from_secs(2);

fn block_compute_worker(runtime: &CoreRuntime) -> SyncSender<()> {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    runtime.handle().task_graph().worker_pool().spawn(move || {
        let _ = started_tx.send(());
        // The sole sender also disconnects this receive if a parent assertion panics.
        let _ = release_rx.recv();
    });
    started_rx
        .recv_timeout(WORKER_START_BUDGET)
        .expect("the real compute worker must start before shutdown");
    release_tx
}

fn assert_graph_joined(report: &TaskGraphShutdownReport) {
    assert!(!report.has_in_flight_work(), "{report:?}");
    assert!(report.timer_joined);
    for kind in [
        TaskPoolKind::Compute,
        TaskPoolKind::AsyncCompute,
        TaskPoolKind::Io,
    ] {
        let workers = report
            .worker_shutdown(kind)
            .expect("every owned worker domain must have an actual census");
        assert!(workers.all_joined(), "{workers:?}");
    }
}

fn assert_compute_blocked(error: &CoreShutdownError) {
    let graph_error = error
        .task_graph_error()
        .expect("the channel-blocked worker must prevent graph completion");
    let report = error
        .graph_report()
        .expect("the failed graph attempt must retain its actual receipt");
    assert_eq!(report, &graph_error.report);
    assert!(report.has_in_flight_work());
    let workers = report.worker_shutdown(TaskPoolKind::Compute).unwrap();
    assert_eq!(workers.active_submission_count, 1);
    assert!(!workers.all_joined());
}

fn cleanup_count(calls: &Arc<Mutex<Vec<String>>>, module: &str) -> usize {
    let expected = format!("{module}:cleanup");
    recorded_calls(calls)
        .iter()
        .filter(|call| *call == &expected)
        .count()
}

fn assert_first_module_failure(error: &CoreShutdownError) {
    let primary = CoreError::MissingConfig("module.cleanup".to_owned());
    assert_eq!(error.module_error(), Some(&primary));
    assert_eq!(
        error.source().unwrap().downcast_ref::<CoreError>(),
        Some(&primary)
    );
    let report = error.module_report();
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
    assert_eq!(
        report.failed,
        vec![("ShutdownDependent".to_owned(), primary)]
    );
    assert_eq!(report.blocked.len(), 1);
    assert_eq!(report.blocked[0].0, "ShutdownProvider");
    assert!(matches!(
        &report.blocked[0].1,
        CoreError::ModuleUnloadBlocked { dependents, .. }
            if dependents == &vec!["ShutdownDependent".to_owned()]
    ));
    assert!(report.not_attempted.is_empty());
    assert!(report.completed_after_deadline.is_empty());
    assert!(!report.deadline_exhausted);
}

#[test]
fn module_failure_retains_successful_graph_receipt_and_retries() {
    let (runtime, calls) = shutdown_fixture();
    let deadline = Instant::now() + SHUTDOWN_BUDGET;
    let error = runtime.shutdown_until(deadline).unwrap_err();
    assert_first_module_failure(&error);
    assert!(error.task_graph_error().is_none());
    assert_graph_joined(
        error
            .graph_report()
            .expect("module failure must not discard the successful graph receipt"),
    );
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
    assert_eq!(cleanup_count(&calls, "ShutdownDependent"), 1);
    assert_eq!(cleanup_count(&calls, "ShutdownProvider"), 0);
    assert_eq!(cleanup_count(&calls, "ShutdownIndependent"), 1);

    let report = runtime
        .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
        .unwrap();
    assert_graph_joined(&report);
    assert!(handle.active_module_shutdown_order().is_empty());
    assert_eq!(cleanup_count(&calls, "ShutdownDependent"), 2);
    assert_eq!(cleanup_count(&calls, "ShutdownProvider"), 1);
    assert_eq!(cleanup_count(&calls, "ShutdownIndependent"), 1);
    let before_repeated_close = recorded_calls(&calls);
    assert_graph_joined(
        &runtime
            .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
            .unwrap(),
    );
    assert_eq!(recorded_calls(&calls), before_repeated_close);
}

#[test]
fn combined_failures_preserve_both_results_until_retry() {
    let (runtime, calls) = shutdown_fixture();
    let blocker = block_compute_worker(&runtime);
    let deadline = Instant::now() + SHUTDOWN_BUDGET;
    let error = runtime.shutdown_until(deadline).unwrap_err();
    assert_first_module_failure(&error);
    assert_compute_blocked(&error);
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("registered module shutdown failed"));
    assert!(diagnostic.contains("owned task graph shutdown failed"));
    assert_eq!(cleanup_count(&calls, "ShutdownDependent"), 1);
    assert_eq!(cleanup_count(&calls, "ShutdownProvider"), 0);
    assert_eq!(cleanup_count(&calls, "ShutdownIndependent"), 1);

    // Retry the original modules while the same actual worker remains blocked.
    let retry_error = runtime
        .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
        .unwrap_err();
    assert!(retry_error.module_error().is_none());
    assert!(retry_error.module_report().is_complete());
    assert_eq!(
        retry_error.module_report().completed,
        vec!["ShutdownDependent", "ShutdownProvider"]
    );
    assert_compute_blocked(&retry_error);
    assert_eq!(
        retry_error
            .source()
            .unwrap()
            .downcast_ref::<TaskGraphShutdownError>(),
        retry_error.task_graph_error()
    );
    assert!(runtime.handle().active_module_shutdown_order().is_empty());
    assert_eq!(cleanup_count(&calls, "ShutdownDependent"), 2);
    assert_eq!(cleanup_count(&calls, "ShutdownProvider"), 1);
    assert_eq!(cleanup_count(&calls, "ShutdownIndependent"), 1);
    let before_graph_retry = recorded_calls(&calls);

    drop(blocker);
    let report = runtime
        .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
        .unwrap();
    assert_graph_joined(&report);
    assert_eq!(recorded_calls(&calls), before_graph_retry);
    // The first error still describes both original failed stages after later success.
    assert_first_module_failure(&error);
    assert_compute_blocked(&error);
}

#[test]
fn expired_deadline_records_both_stages_without_cleanup() {
    let (runtime, calls) = shutdown_fixture();
    let handle = runtime.handle();
    let before_order = handle.active_module_shutdown_order();
    let before_calls = recorded_calls(&calls);
    let blocker = block_compute_worker(&runtime);
    let deadline = Instant::now();
    let error = runtime.shutdown_until(deadline).unwrap_err();
    let modules = error.module_report();
    assert!(!modules.is_complete());
    assert!(modules.deadline_exhausted);
    assert!(modules.attempted.is_empty());
    assert!(modules.completed.is_empty());
    assert!(modules.completed_after_deadline.is_empty());
    assert!(modules.failed.is_empty());
    assert!(modules.blocked.is_empty());
    assert_eq!(
        modules.not_attempted,
        before_order.iter().rev().cloned().collect::<Vec<_>>()
    );
    assert!(matches!(
        error.module_error(),
        Some(CoreError::ModuleCleanupTimeout {
            module,
            incomplete_entries: 3,
            failed: 0,
            cancelled: 0,
            ..
        }) if module == "ShutdownDependent"
    ));
    assert_eq!(
        error.source().unwrap().downcast_ref::<CoreError>(),
        error.module_error()
    );
    assert_compute_blocked(&error);
    assert_eq!(recorded_calls(&calls), before_calls);
    assert_eq!(handle.active_module_shutdown_order(), before_order);
    {
        let modules = handle.inner.modules.lock().unwrap();
        for module in &before_order {
            assert_eq!(
                modules.get(module).unwrap().lifecycle,
                LifecycleState::Running
            );
        }
    }

    drop(blocker);
    let first_retry = runtime
        .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
        .unwrap_err();
    assert_first_module_failure(&first_retry);
    assert!(first_retry.task_graph_error().is_none());
    assert_graph_joined(first_retry.graph_report().unwrap());
    let report = runtime
        .shutdown_until(Instant::now() + SHUTDOWN_BUDGET)
        .unwrap();
    assert_graph_joined(&report);
    assert!(handle.active_module_shutdown_order().is_empty());
    assert_eq!(cleanup_count(&calls, "ShutdownDependent"), 2);
    assert_eq!(cleanup_count(&calls, "ShutdownProvider"), 1);
    assert_eq!(cleanup_count(&calls, "ShutdownIndependent"), 1);
}
