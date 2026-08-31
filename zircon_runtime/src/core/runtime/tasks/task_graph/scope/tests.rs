use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::Duration;

use super::*;
use crate::core::runtime::tasks::{
    EngineTaskGraph, EngineTaskGraphOptions, JobScheduler, TaskDescriptor, TaskId, TaskPool,
    TaskPoolDescriptor, TaskPoolKind, TaskState,
};

fn descriptor(id: u64, policy: TaskCancellationPolicy) -> TaskDescriptor {
    TaskDescriptor::new(TaskId::new(id), TaskPoolKind::Compute, "test")
        .with_cancellation_policy(policy)
}

#[test]
fn scoped_submission_binds_descriptor_status_and_wait_to_one_handle() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("canonical-handle"))
        .expect("running runtime should admit a scope");
    let task = scope
        .submit(
            descriptor(42, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("scoped task should be admitted");

    assert_eq!(task.descriptor().id, TaskId::new(42));
    task.wait();
    assert_eq!(task.status().state, TaskState::Completed);
    assert!(task.is_complete());

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("completed canonical task should drain");
}

#[test]
fn canonical_status_never_precedes_or_lags_completion_fence_under_race() {
    const TASK_COUNT: u64 = 512;
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(2))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("canonical-race"))
        .expect("running runtime should admit a scope");

    for id in 1..=TASK_COUNT {
        let task = scope
            .submit(
                descriptor(1_000 + id, TaskCancellationPolicy::FinishOnShutdown),
                |_| std::thread::yield_now(),
            )
            .expect("race probe should be admitted");

        loop {
            let status = task.status();
            if status.is_terminal() {
                assert!(
                    task.is_complete(),
                    "terminal status must be committed by the completion authority"
                );
                break;
            }
            if task.is_complete() {
                assert!(
                    task.status().is_terminal(),
                    "completion and status must share one monotonic lifecycle state"
                );
                break;
            }
            std::thread::yield_now();
        }
        task.wait();
    }

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("race probes should drain");
}

#[test]
fn dependency_wiring_retains_cancel_on_drop_prerequisite_until_terminal() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let prerequisite_scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("retained-prerequisite"))
        .expect("running runtime should admit prerequisite scope");
    let dependent_scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("retained-dependent"))
        .expect("running runtime should admit dependent scope");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (blocker_release_tx, blocker_release_rx) = mpsc::sync_channel(0);
    scheduler.schedule(move || {
        blocker_started_tx.send(()).expect("blocker should start");
        blocker_release_rx.recv().expect("blocker should release");
    });
    blocker_started_rx
        .recv()
        .expect("worker should be occupied");

    let prerequisite = prerequisite_scope
        .schedule(
            &scheduler,
            descriptor(50, TaskCancellationPolicy::CancelOnDrop),
            |_| {},
        )
        .expect("prerequisite should be admitted");
    let (dependent_ran_tx, dependent_ran_rx) = mpsc::sync_channel(1);
    let dependent = dependent_scope
        .schedule_after(
            &scheduler,
            &[prerequisite.clone()],
            descriptor(51, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                dependent_ran_tx
                    .send(())
                    .expect("dependent should run after its retained prerequisite");
            },
        )
        .expect("dependent should be admitted");
    drop(prerequisite);

    blocker_release_tx.send(()).expect("worker should release");
    dependent.wait();
    dependent_ran_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("dependency wiring must keep the prerequisite alive");
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("retained dependency chain should drain");
}

#[test]
fn detached_cancellation_keeps_status_and_dependency_fence_consistent() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (blocker_release_tx, blocker_release_rx) = mpsc::sync_channel(0);
    scheduler.schedule(move || {
        blocker_started_tx.send(()).expect("blocker should start");
        blocker_release_rx.recv().expect("blocker should release");
    });
    blocker_started_rx
        .recv()
        .expect("worker should be occupied");

    let task = TaskHandle::schedule_detached(
        &scheduler,
        descriptor(60, TaskCancellationPolicy::CancelOnDrop),
        |_| panic!("cancelled detached work must not run"),
    );
    task.request_cancellation();
    blocker_release_tx.send(()).expect("worker should release");
    task.wait();

    assert_eq!(task.status().state, TaskState::Cancelled);
    assert_eq!(task.completion.terminal_state(), Some(TaskState::Cancelled));
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("cancelled detached task should drain");
}

#[test]
fn detached_panic_keeps_status_and_dependency_fence_consistent() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let task = TaskHandle::schedule_detached(
        &scheduler,
        descriptor(61, TaskCancellationPolicy::FinishOnShutdown),
        |_| panic!("detached failure"),
    );

    assert!(catch_unwind(AssertUnwindSafe(|| task.wait())).is_err());
    assert_eq!(task.status().state, TaskState::Failed);
    assert_eq!(task.completion.terminal_state(), Some(TaskState::Failed));
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("panicked detached task should still retire its worker lease");
}

#[test]
fn late_terminal_observer_runs_after_task_graph_workers_join() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("late-observer"))
        .expect("running runtime should admit a scope");
    let task = scope
        .submit(
            descriptor(62, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("task should be admitted");
    task.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("runtime should join before late registration");

    let (observed_tx, observed_rx) = mpsc::sync_channel(1);
    task.on_terminal(move || {
        observed_tx.send(()).expect("observer should publish");
    });
    observed_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("late observer must not require a live or process-default worker owner");
}

#[test]
fn close_admission_cancels_queued_cancel_on_drop_work_before_user_code_runs() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("test"))
        .expect("running runtime should admit a scope");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    scope
        .submit(
            descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                started_task.wait();
                release_task.wait();
            },
        )
        .expect("first task should be admitted");
    started.wait();
    let cancelled = scope
        .submit(
            descriptor(2, TaskCancellationPolicy::CancelOnDrop),
            move |_| {
                result_tx.send(()).expect("cancelled task must not run");
            },
        )
        .expect("queued task should be admitted");

    scope.close_admission();
    release.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("scope must drain after its queued cancellation reaches a worker");

    assert_eq!(cancelled.status().state, TaskState::Cancelled);
    assert!(result_rx.try_recv().is_err());
}

#[test]
fn dropping_the_last_cancel_on_drop_handle_cancels_queued_work() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("drop-cancel"))
        .expect("running runtime should admit a scope");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    scope
        .submit(
            descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                started_task.wait();
                release_task.wait();
            },
        )
        .expect("blocking task should be admitted");
    started.wait();

    let (ran_tx, ran_rx) = mpsc::sync_channel(1);
    let task = scope
        .submit(
            descriptor(2, TaskCancellationPolicy::CancelOnDrop),
            move |_| {
                ran_tx
                    .send(())
                    .expect("dropped cancel-on-drop work must not run");
            },
        )
        .expect("queued task should be admitted");
    let retained_handle = task.clone();
    drop(task);
    assert!(!retained_handle.is_cancellation_requested());
    drop(retained_handle);

    release.wait();
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while scope.census().cancelled == 0 && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("dropped queued work should already have retired from its scope");

    let census = scope.census();
    assert_eq!(census.cancelled, 1);
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);
    assert!(ran_rx.try_recv().is_err());
}

#[test]
fn dropping_the_last_scope_handle_closes_admission_while_task_handles_remain() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("scope-drop"))
        .expect("running runtime should admit a scope");
    let task = scope
        .submit(
            descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("scoped task should be admitted");
    let retained_scope = scope.clone();

    drop(scope);

    assert!(task.scope_census().accepting);
    drop(retained_scope);

    assert!(!task.scope_census().accepting);
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("closing the last scope handle must preserve a drainable runtime");
}

#[test]
fn shutdown_timeout_keeps_the_runtime_closing_and_reports_the_running_scope() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("blocking"))
        .expect("running runtime should admit a scope");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    scope
        .submit(
            descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                started_task.wait();
                release_task.wait();
            },
        )
        .expect("blocking task should be admitted");
    started.wait();

    let error = runtime
        .shutdown(Duration::ZERO)
        .expect_err("running work must make the shutdown deadline incomplete");
    assert_eq!(error.report.scopes.len(), 1);
    assert_eq!(error.report.scopes[0].owner, "blocking");
    assert_eq!(error.report.scopes[0].running, 1);
    release.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("a closing runtime should permit a later drain attempt");
}

#[test]
fn scheduler_submission_uses_scope_cancellation_and_drain_accounting() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("scheduled"))
        .expect("running runtime should admit a scope");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    scheduler.schedule(move || {
        started_task.wait();
        release_task.wait();
    });
    started.wait();

    let (ran_tx, ran_rx) = mpsc::sync_channel(1);
    let completion = scope
        .schedule(
            &scheduler,
            descriptor(3, TaskCancellationPolicy::CancelOnDrop),
            move |_| {
                ran_tx
                    .send(())
                    .expect("cancelled scheduled work must not run");
            },
        )
        .expect("scope-owned scheduler work should be admitted");

    scope.close_admission();
    release.wait();
    completion.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("scope-owned scheduled work should drain");

    assert!(completion.is_cancelled());
    assert_eq!(scope.census().cancelled, 1);
    assert!(ran_rx.try_recv().is_err());
}

#[test]
fn running_work_that_ignores_a_cancellation_request_completes() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("ignored-cancellation"))
        .expect("running runtime should admit a scope");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    let task = scope
        .submit(
            descriptor(9, TaskCancellationPolicy::CancelOnDrop),
            move |_| {
                started_task.wait();
                release_task.wait();
            },
        )
        .expect("running work should be admitted");
    started.wait();

    task.request_cancellation();
    release.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("normally returned work should drain");

    assert_eq!(task.status().state, TaskState::Completed);
    assert_eq!(scope.census().completed, 1);
    assert_eq!(scope.census().cancelled, 0);
}

#[test]
fn running_work_reports_cancelled_only_after_acknowledgement() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("acknowledged-cancellation"))
        .expect("running runtime should admit a scope");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_task = Arc::clone(&started);
    let release_task = Arc::clone(&release);
    let task = scope
        .submit(
            descriptor(10, TaskCancellationPolicy::CancelOnDrop),
            move |token| {
                started_task.wait();
                release_task.wait();
                assert!(token.acknowledge_cancellation());
            },
        )
        .expect("running work should be admitted");
    started.wait();

    task.request_cancellation();
    release.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("acknowledged cancellation should drain");

    assert_eq!(task.status().state, TaskState::Cancelled);
    assert_eq!(scope.census().completed, 0);
    assert_eq!(scope.census().cancelled, 1);
}

#[test]
fn scheduler_dependency_failure_retires_scope_admission_without_running_user_work() {
    let mut runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("dependent"))
        .expect("running runtime should admit a scope");
    let prerequisite_scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("prerequisite"))
        .expect("running runtime should admit a prerequisite scope");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let dependency = prerequisite_scope
        .schedule(
            &scheduler,
            descriptor(7, TaskCancellationPolicy::FinishOnShutdown),
            |_| panic!("dependency failure"),
        )
        .expect("scope should admit the failing prerequisite");
    let (ran_tx, ran_rx) = mpsc::sync_channel(1);

    let scheduled = scope
        .schedule_after(
            &scheduler,
            &[dependency],
            descriptor(8, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                ran_tx
                    .send(())
                    .expect("failed dependencies must not run scoped user work");
            },
        )
        .expect("scope-owned dependent work should be admitted");

    assert!(catch_unwind(AssertUnwindSafe(|| scheduled.wait())).is_err());
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("a dependency failure must still retire the scoped admission");

    let census = scope.census();
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);
    assert_eq!(census.failed, 1);
    assert!(ran_rx.try_recv().is_err());
}

#[test]
fn scheduler_submission_rejects_closed_full_and_unrelated_pool_scopes() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("guarded").with_task_capacity(1))
        .expect("running runtime should admit a scope");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let scheduled = scope
        .schedule(
            &scheduler,
            descriptor(4, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                started_tx.send(()).expect("scoped task should start");
                release_rx.recv().expect("scoped task should release");
            },
        )
        .expect("first scope task should be admitted");
    started_rx
        .recv()
        .expect("scope task should occupy capacity");
    assert!(matches!(
        scope.schedule(
            &scheduler,
            descriptor(5, TaskCancellationPolicy::CancelOnDrop),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::ScopeCapacityReached { .. })
    ));

    let unrelated = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    assert!(matches!(
        scope.schedule(
            &unrelated,
            descriptor(6, TaskCancellationPolicy::CancelOnDrop),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::SchedulerOwnerMismatch { .. })
    ));

    scope.close_admission();
    assert!(matches!(
        scope.schedule(
            &scheduler,
            descriptor(7, TaskCancellationPolicy::CancelOnDrop),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::ScopeClosed { .. })
    ));
    release_tx.send(()).expect("scoped task should release");
    scheduled.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("finished task should drain");
}
