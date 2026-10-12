use super::super::EngineTaskGraphOptions;
use super::EngineTaskGraph;
use crate::core::runtime::tasks::{
    TaskDescriptor, TaskGraphWorkerShutdownCensus, TaskId, TaskPoolKind,
};
use crate::core::CoreError;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

#[test]
fn worker_inventory_reports_three_domains_under_one_exact_global_budget() {
    for (worker_count, expected_total, expected_domains) in
        [(1, 3, [1, 1, 1]), (2, 3, [1, 1, 1]), (7, 7, [2, 2, 3])]
    {
        let graph =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(worker_count))
                .expect("task graph should distribute one runtime worker budget");

        let inventory = graph.worker_inventory();

        assert_eq!(inventory.worker_count(), expected_total);
        assert_eq!(inventory.worker_set_count(), 3);
        for (kind, expected_workers, expected_name) in [
            (TaskPoolKind::Io, expected_domains[0], "zircon-io-task"),
            (
                TaskPoolKind::AsyncCompute,
                expected_domains[1],
                "zircon-async-compute-task",
            ),
            (
                TaskPoolKind::Compute,
                expected_domains[2],
                "zircon-compute-task",
            ),
        ] {
            let domain = inventory.domain(kind).expect("worker domain should exist");
            assert_eq!(domain.worker_count, expected_workers);
            assert_eq!(domain.thread_name, expected_name);
        }
    }
}

#[test]
fn dropping_empty_scopes_retires_graph_registration_immediately() {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");

    for index in 0..1_024 {
        let scope = graph
            .create_scope(super::TaskGraphScopeDescriptor::new(format!(
                "scope-{index}"
            )))
            .expect("running task graph should admit a scope");
        drop(scope);
    }

    assert_eq!(graph.inner.lock_state().scopes.len(), 0);
}

#[test]
fn shutdown_joins_owned_workers_even_when_pool_handles_are_retained() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("task graph should create its worker budget");
    let retained_pool_handle = runtime.worker_pool().clone();

    let report = runtime
        .shutdown(Duration::from_secs(2))
        .expect("an idle runtime should join every owned worker");

    assert_eq!(report.worker_shutdowns.len(), 3);
    assert!(report
        .worker_shutdowns
        .iter()
        .all(TaskGraphWorkerShutdownCensus::all_joined));
    let compute = report
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("compute shutdown census should be reported");
    assert!(compute.termination_signalled);
    assert_eq!(compute.expected_worker_count, 1);
    assert_eq!(compute.exited_worker_count, 1);
    assert_eq!(compute.joined_worker_count, 1);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        retained_pool_handle.spawn(|| {});
    }))
    .is_err());

    let repeated = runtime
        .shutdown(Duration::from_secs(2))
        .expect("repeated shutdown should preserve the joined receipt");
    assert!(repeated
        .worker_shutdowns
        .iter()
        .all(TaskGraphWorkerShutdownCensus::all_joined));
    drop(retained_pool_handle);
}

#[test]
fn owned_timer_shutdown_is_independent_for_two_task_graphs() {
    let first = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("first graph should start");
    let second = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("second graph should start");
    let first_timer = first
        .lifecycle_timer()
        .expect("first graph timer should start");
    let second_timer = second
        .lifecycle_timer()
        .expect("second graph timer should start");

    let first_report = first
        .shutdown(Duration::from_secs(1))
        .expect("first graph should join its timer and pools");
    assert!(first_report.timer_joined);
    assert!(first_timer.schedule_at(Instant::now(), || {}).is_err());

    let (delivered_tx, delivered_rx) = mpsc::sync_channel(1);
    let _subscription = second_timer
        .schedule_at(Instant::now(), move || {
            delivered_tx
                .send(())
                .expect("second graph callback delivered");
        })
        .expect("second graph timer must remain open");
    delivered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("second graph callback should run after first graph stops");
    let second_report = second
        .shutdown(Duration::from_secs(1))
        .expect("second graph should join independently");
    assert!(second_report.timer_joined);
}

#[test]
fn owned_timer_spawn_failure_is_typed_and_can_retry_before_graph_close() {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("graph should start without a timer worker");
    let error = graph
        .lifecycle_timer_with(|_| Err(CoreError::ThreadSpawn("fixture".to_string())))
        .expect_err("first timer spawn should fail");
    assert_eq!(error, CoreError::ThreadSpawn("fixture".to_string()));

    let timer = graph
        .lifecycle_timer()
        .expect("transient timer spawn failure should permit a retry");
    let report = graph
        .shutdown(Duration::from_secs(1))
        .expect("retried timer and graph pools should join");
    assert!(report.timer_started);
    assert!(report.timer_joined);
    assert!(timer.schedule_at(Instant::now(), || {}).is_err());
    assert_eq!(
        graph
            .lifecycle_timer()
            .expect_err("closed graph must reject timer access"),
        CoreError::RuntimeUnavailable
    );
}

#[test]
fn blocked_owned_timer_callback_keeps_graph_shutdown_incomplete_until_retry() {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("graph should start");
    let timer = graph.lifecycle_timer().expect("graph timer should start");
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let release = Arc::new(Barrier::new(2));
    let release_callback = Arc::clone(&release);
    let _subscription = timer
        .schedule_at(Instant::now(), move || {
            started_tx.send(()).expect("callback should report start");
            release_callback.wait();
        })
        .expect("graph timer should admit callback");
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("callback should occupy graph async workers");

    let timeout = graph
        .shutdown(Duration::from_millis(20))
        .expect_err("blocked callback must prevent graph completion");
    assert!(timeout.report.has_in_flight_work());
    assert!(timeout.report.timer_joined);
    assert!(timer.schedule_at(Instant::now(), || {}).is_err());
    release.wait();

    let retry = graph
        .shutdown(Duration::from_secs(1))
        .expect("released callback and timer worker should join");
    assert!(retry.timer_joined);
    assert!(retry
        .worker_shutdowns
        .iter()
        .all(TaskGraphWorkerShutdownCensus::all_joined));
}

#[test]
fn shutdown_timeout_keeps_unjoined_workers_visible_and_retryable() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("task graph should create its worker budget");
    let worker_pool = runtime.worker_pool().clone();
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    worker_pool.spawn(move || {
        started_sender
            .send(())
            .expect("test owner should observe worker start");
        release_receiver
            .recv()
            .expect("test owner should release blocked worker");
    });
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("unscoped worker task should start");

    let timeout = runtime
        .shutdown(Duration::ZERO)
        .expect_err("a blocked unscoped worker must prevent a joined receipt");
    let workers = timeout
        .report
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("blocked compute domain should remain visible");
    assert_eq!(workers.active_submission_count, 1);
    assert!(!workers.termination_signalled);
    assert!(workers.joined_worker_count < workers.expected_worker_count);
    assert!(timeout.report.has_in_flight_work());

    release_sender
        .send(())
        .expect("blocked worker should still be owned by the closing runtime");
    let retry = runtime
        .shutdown(Duration::from_secs(2))
        .expect("shutdown retry should join the released worker");
    let workers = retry
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("compute retry census should remain visible");
    assert!(workers.termination_signalled);
    assert_eq!(workers.active_submission_count, 0);
    assert!(workers.all_joined());
}

#[test]
fn astra_life_a1_shutdown_retry_and_repeat_preserve_scope_census_after_handles_drop() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(super::TaskGraphScopeDescriptor::new("retry-census"))
        .expect("running task graph should admit a scope");
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let task = scope
        .submit(
            TaskDescriptor::new(TaskId::new(1), TaskPoolKind::Compute, "retry-census"),
            move |_| {
                started_sender
                    .send(())
                    .expect("test owner should observe scope task start");
                release_receiver
                    .recv()
                    .expect("test owner should release scope task");
            },
        )
        .expect("scope should admit its blocking task");
    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("scope task should start");

    let timeout = runtime
        .shutdown(Duration::ZERO)
        .expect_err("blocked scope task should exhaust a zero shutdown budget");
    assert_eq!(timeout.report.scopes.len(), 1);
    assert_eq!(timeout.report.scopes[0].owner, "retry-census");
    assert_eq!(timeout.report.scopes[0].submitted, 1);
    assert_eq!(timeout.report.scopes[0].running, 1);

    release_sender
        .send(())
        .expect("closing graph should retain the admitted scope task");
    task.wait();
    drop(task);
    drop(scope);

    let retry = runtime
        .shutdown(Duration::from_secs(2))
        .expect("shutdown retry should retain and finish the original scope census");
    assert_eq!(retry.scopes.len(), 1);
    assert_eq!(retry.scopes[0].owner, "retry-census");
    assert_eq!(retry.scopes[0].submitted, 1);
    assert_eq!(retry.scopes[0].running, 0);
    assert_eq!(retry.scopes[0].completed, 1);

    let repeated = runtime
        .shutdown(Duration::from_secs(2))
        .expect("repeated shutdown should preserve the final scope census");
    assert_eq!(repeated.scopes, retry.scopes);
    assert!(runtime.inner.lock_state().shutdown_scopes.is_empty());
}

#[test]
fn shutdown_until_does_not_restart_budget_for_later_scope_drains() {
    const FIRST_SCOPE_RELEASE_DELAY: Duration = Duration::from_millis(100);
    const SHARED_SHUTDOWN_BUDGET: Duration = Duration::from_millis(500);
    const MAX_SHARED_SHUTDOWN_ELAPSED: Duration = Duration::from_millis(575);

    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("task graph should create its worker budget");
    let first_scope = runtime
        .create_scope(super::TaskGraphScopeDescriptor::new("first-deadline-stage"))
        .expect("running task graph should admit the first scope");
    let second_scope = runtime
        .create_scope(super::TaskGraphScopeDescriptor::new(
            "second-deadline-stage",
        ))
        .expect("running task graph should admit the second scope");

    let (first_started_sender, first_started_receiver) = mpsc::sync_channel(1);
    let (first_release_sender, first_release_receiver) = mpsc::sync_channel(1);
    let first_task = first_scope
        .submit(
            TaskDescriptor::new(
                TaskId::new(1),
                TaskPoolKind::Compute,
                "first-deadline-stage",
            ),
            move |_| {
                first_started_sender
                    .send(())
                    .expect("test owner should observe first scope task start");
                first_release_receiver
                    .recv()
                    .expect("test owner should release the first scope task");
            },
        )
        .expect("first scope should admit its blocking task");
    let (second_started_sender, second_started_receiver) = mpsc::sync_channel(1);
    let (second_release_sender, second_release_receiver) = mpsc::sync_channel(1);
    let second_task = second_scope
        .submit(
            TaskDescriptor::new(TaskId::new(2), TaskPoolKind::Io, "second-deadline-stage"),
            move |_| {
                second_started_sender
                    .send(())
                    .expect("test owner should observe second scope task start");
                second_release_receiver
                    .recv()
                    .expect("test owner should release the second scope task");
            },
        )
        .expect("second scope should admit its blocking task");
    first_started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("first scope task should start");
    second_started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("second scope task should start");

    let first_scope_for_release = first_scope.clone();
    let release_first_scope = std::thread::spawn(move || {
        let close_deadline = Instant::now() + Duration::from_secs(2);
        while first_scope_for_release.census().accepting {
            assert!(
                Instant::now() < close_deadline,
                "shutdown should close first scope admission before draining it"
            );
            std::thread::yield_now();
        }
        std::thread::sleep(FIRST_SCOPE_RELEASE_DELAY);
        first_release_sender
            .send(())
            .expect("first scope task should still be owned during shutdown");
    });
    let shutdown_started_at = Instant::now();
    let timeout = runtime.shutdown_until(shutdown_started_at + SHARED_SHUTDOWN_BUDGET);
    let shutdown_elapsed = shutdown_started_at.elapsed();

    release_first_scope
        .join()
        .expect("first release thread should finish");
    second_release_sender
        .send(())
        .expect("second scope task should still be owned after the shared deadline");
    first_task.wait();
    second_task.wait();
    drop(first_task);
    drop(second_task);
    drop(first_scope);
    drop(second_scope);

    let timeout = timeout.expect_err(
        "the second scope must not receive a fresh shutdown duration after the first consumes budget",
    );
    assert_eq!(timeout.report.scopes.len(), 2);
    assert_eq!(timeout.report.scopes[0].owner, "first-deadline-stage");
    assert!(timeout.report.scopes[0].is_quiescent());
    assert_eq!(timeout.report.scopes[1].owner, "second-deadline-stage");
    assert!(!timeout.report.scopes[1].is_quiescent());
    assert!(
        shutdown_elapsed >= FIRST_SCOPE_RELEASE_DELAY,
        "shutdown returned before the first scope consumed its staged delay: elapsed={shutdown_elapsed:?}"
    );
    assert!(
        shutdown_elapsed < MAX_SHARED_SHUTDOWN_ELAPSED,
        "shutdown restarted a full stage budget: elapsed={shutdown_elapsed:?}"
    );

    let retry = runtime
        .shutdown(Duration::from_secs(2))
        .expect("released scopes should remain retryable after the shared deadline expires");
    assert!(retry.scopes.iter().all(|scope| scope.is_quiescent()));
    assert!(retry
        .worker_shutdowns
        .iter()
        .all(TaskGraphWorkerShutdownCensus::all_joined));
}

#[test]
fn shutdown_waits_for_accepted_terminal_observers_before_worker_quit() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(super::TaskGraphScopeDescriptor::new("terminal-observer"))
        .expect("running task graph should admit a scope");
    let scheduled = scope
        .submit(
            crate::core::runtime::tasks::TaskDescriptor::new(
                crate::core::runtime::tasks::TaskId::new(1),
                crate::core::runtime::tasks::TaskPoolKind::Compute,
                "terminal-observer",
            ),
            |_| {},
        )
        .expect("scope should accept submitted work");
    let (observer_started_tx, observer_started_rx) = mpsc::sync_channel(1);
    let (observer_release_tx, observer_release_rx) = mpsc::sync_channel(1);
    scheduled.on_terminal(move || {
        observer_started_tx
            .send(())
            .expect("test owner should observe terminal callback start");
        observer_release_rx
            .recv()
            .expect("test owner should release terminal callback");
    });
    scheduled.wait();
    observer_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("accepted terminal callback should reach the worker");

    let timeout = runtime
        .shutdown(Duration::ZERO)
        .expect_err("active terminal callback must keep its execution lease visible");
    let workers = timeout
        .report
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("callback compute domain should remain visible");
    assert_eq!(workers.active_submission_count, 1);
    assert!(!workers.termination_signalled);

    observer_release_tx
        .send(())
        .expect("terminal callback should remain owned after timeout");
    let retry = runtime
        .shutdown(Duration::from_secs(1))
        .expect("shutdown retry should wait for callback completion and join");
    let workers = retry
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("compute retry census should remain visible");
    assert_eq!(workers.active_submission_count, 0);
    assert!(workers.all_joined());
}

#[test]
fn shutdown_from_owned_worker_returns_incomplete_without_self_joining() {
    let runtime = Arc::new(
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("task graph should create its worker budget"),
    );
    let worker_runtime = Arc::clone(&runtime);
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    runtime.worker_pool().spawn(move || {
        let error = worker_runtime
            .shutdown(Duration::from_secs(30))
            .expect_err("a worker cannot publish a receipt that joins itself");
        result_sender
            .send(error.report)
            .expect("test owner should receive the incomplete receipt");
    });

    let report = result_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("worker-side shutdown must not wait for its own deadline");
    assert!(!report
        .worker_shutdown(TaskPoolKind::Compute)
        .expect("worker-side shutdown should retain compute census")
        .all_joined());

    let retry = runtime
        .shutdown(Duration::from_secs(2))
        .expect("external retry should join the worker after its task returns");
    assert!(retry
        .worker_shutdowns
        .iter()
        .all(TaskGraphWorkerShutdownCensus::all_joined));
}
