//! Models from Unreal Core/Tests/Tasks/TasksTest.cpp:
//! FTasksBasicTest (:187, fire-and-forget) and FTasksPipeWaitUntilEmptyTest
//! (:680, outstanding work prevents successful draining).
//! UE has no matching DetachOnDrop/FinishOnShutdown enum distinction here. These
//! cases preserve their shared completion behavior without settling R01-Q1.

use super::*;

const WAIT: Duration = Duration::from_secs(5);
const FIRE_AND_FORGET_POLICIES: [TaskCancellationPolicy; 2] = [
    TaskCancellationPolicy::DetachOnDrop,
    TaskCancellationPolicy::FinishOnShutdown,
];

#[test]
fn unreal_equiv_r01_fire_and_forget_survives_last_handle_release_while_queued() {
    for policy in FIRE_AND_FORGET_POLICIES {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("graph should own one worker per domain");
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("r01-handle-release"))
            .expect("running graph should admit a scope");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let blocker = scope
            .submit(
                descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
                move |_| {
                    started_tx.send(()).expect("owner should observe the gate");
                    release_rx
                        .recv_timeout(WAIT)
                        .expect("owner must open the gate");
                },
            )
            .expect("blocking task should be admitted");
        started_rx
            .recv_timeout(WAIT)
            .expect("compute worker should reach the gate");
        let (ran_tx, ran_rx) = mpsc::sync_channel(1);
        let task = scope
            .submit(descriptor(2, policy), move |token| {
                ran_tx
                    .send(token.is_cancellation_requested())
                    .expect("owner should observe execution");
            })
            .expect("fire-and-forget work should be admitted behind the gate");

        drop(task);
        assert_eq!(scope.census().queued, 1, "{policy:?}");
        assert!(ran_rx.try_recv().is_err());
        release_tx
            .send(())
            .expect("compute gate should still be live");
        let report = runtime
            .shutdown(WAIT)
            .expect("unreferenced work must still drain");

        assert!(!report.has_in_flight_work());
        assert_eq!(blocker.status().state, TaskState::Completed);
        assert!(!ran_rx
            .try_recv()
            .expect("dropped handle must not suppress execution"));
        let census = scope.census();
        assert_eq!(census.completed, 2, "{policy:?}");
        assert_eq!(census.cancelled, 0, "{policy:?}");
        assert_eq!(census.failed, 0, "{policy:?}");
    }
}

#[test]
fn unreal_equiv_r01_scope_close_preserves_queued_fire_and_forget_work() {
    for policy in FIRE_AND_FORGET_POLICIES {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("graph should own one compute worker");
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("r01-queued-close"))
            .expect("running graph should admit a scope");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let blocker = scope
            .submit(
                descriptor(1, TaskCancellationPolicy::FinishOnShutdown),
                move |_| {
                    started_tx.send(()).expect("owner should observe the gate");
                    release_rx
                        .recv_timeout(WAIT)
                        .expect("owner must open the gate");
                },
            )
            .expect("blocking task should be admitted");
        started_rx
            .recv_timeout(WAIT)
            .expect("compute worker should reach the gate");
        let (ran_tx, ran_rx) = mpsc::sync_channel(1);
        let task = scope
            .submit(descriptor(2, policy), move |token| {
                ran_tx
                    .send(token.is_cancellation_requested())
                    .expect("owner should observe execution");
            })
            .expect("queued task should be admitted");

        scope.close_admission();
        assert!(!task.is_cancellation_requested(), "{policy:?}");
        assert!(!scope.wait_until_quiescent(Duration::ZERO));
        let pending = runtime
            .shutdown(Duration::ZERO)
            .expect_err("blocked work must prevent a successful drain");
        assert_eq!(pending.report.scopes[0].queued, 1, "{policy:?}");
        assert_eq!(pending.report.scopes[0].running, 1, "{policy:?}");
        release_tx
            .send(())
            .expect("compute gate should still be live");
        let report = runtime
            .shutdown(WAIT)
            .expect("closing graph should drain after gate release");

        assert!(!report.has_in_flight_work());
        assert_eq!(blocker.status().state, TaskState::Completed);
        assert_eq!(task.status().state, TaskState::Completed, "{policy:?}");
        assert!(!ran_rx
            .try_recv()
            .expect("scope closure must preserve queued work"));
        assert_eq!(scope.census().completed, 2, "{policy:?}");
        assert_eq!(scope.census().cancelled, 0, "{policy:?}");
    }
}

#[test]
fn unreal_equiv_r01_scope_close_waits_for_running_unreferenced_work() {
    for policy in FIRE_AND_FORGET_POLICIES {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("graph should own its worker domains");
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("r01-running-close"))
            .expect("running graph should admit a scope");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let (ran_tx, ran_rx) = mpsc::sync_channel(1);
        let task = scope
            .submit(descriptor(1, policy), move |token| {
                started_tx
                    .send(())
                    .expect("owner should observe running work");
                release_rx
                    .recv_timeout(WAIT)
                    .expect("owner must release running work");
                ran_tx
                    .send(token.is_cancellation_requested())
                    .expect("owner should observe completion");
            })
            .expect("task should be admitted");
        started_rx
            .recv_timeout(WAIT)
            .expect("task should start before its handle is dropped");

        drop(task);
        scope.close_admission();
        let pending = runtime
            .shutdown(Duration::ZERO)
            .expect_err("handle release and scope closure are not completion");
        assert!(pending.report.has_in_flight_work());
        assert_eq!(pending.report.scopes[0].running, 1, "{policy:?}");
        release_tx
            .send(())
            .expect("running work should still retain its gate");
        let report = runtime
            .shutdown(WAIT)
            .expect("retry must drain the original running task");

        assert!(!report.has_in_flight_work());
        assert!(!ran_rx
            .try_recv()
            .expect("unreferenced work must still finish"));
        assert_eq!(scope.census().completed, 1, "{policy:?}");
        assert_eq!(scope.census().cancelled, 0, "{policy:?}");
    }
}
