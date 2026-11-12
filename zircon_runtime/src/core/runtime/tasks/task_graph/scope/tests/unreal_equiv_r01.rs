//! Models from Unreal Core/Tests/Tasks/TasksTest.cpp:
//! FTasksBasicTest (:187, fire-and-forget) and FTasksPipeWaitUntilEmptyTest
//! (:680, outstanding work prevents successful draining; the prerequisite
//! sub-case is :688-717).
//! UE has no matching DetachOnDrop/FinishOnShutdown enum distinction here. These
//! cases preserve their shared completion behavior without settling R01-Q1.

use super::*;

const WAIT: Duration = Duration::from_secs(5);
// UE's WaitUntilEmpty/Wait budget in the prerequisite sub-case.
const UE_PIPE_WAIT: Duration = Duration::from_millis(100);
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
            .expect("graph should own one worker per domain");
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

#[test]
fn unreal_equiv_r01_scope_stays_busy_while_admitted_work_awaits_its_prerequisite() {
    for policy in FIRE_AND_FORGET_POLICIES {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("graph should own one worker per domain");
        // UE's FTaskEvent is not pipe work. Its stand-in is a gated task owned by
        // another scope and another worker domain, so the scope under test only
        // accounts for the two launched tasks and its compute worker stays free.
        let event_scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("r01-prerequisite-event"))
            .expect("running graph should admit the prerequisite scope");
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("r01-prerequisite-pipe"))
            .expect("running graph should admit a scope");
        assert!(scope.wait_until_quiescent(Duration::ZERO), "{policy:?}");

        let (trigger_tx, trigger_rx) = mpsc::sync_channel::<()>(1);
        let prerequisite = event_scope
            .submit(
                TaskDescriptor::new(TaskId::new(3), TaskPoolKind::Io, "r01-prerequisite-event")
                    .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown),
                move |_| {
                    trigger_rx
                        .recv_timeout(WAIT)
                        .expect("owner must trigger the prerequisite");
                },
            )
            .expect("prerequisite stand-in should be admitted");
        let task1 = scope
            .submit_after(
                std::slice::from_ref(&prerequisite),
                descriptor(1, policy),
                |_| {},
            )
            .expect("work with a pending prerequisite should be admitted");

        // The scope knows about the task even though its prerequisite is pending.
        assert_eq!(scope.census().queued, 1, "{policy:?}");
        assert!(!task1.is_complete(), "{policy:?}");
        assert!(!scope.wait_until_quiescent(UE_PIPE_WAIT), "{policy:?}");

        let (ran_tx, ran_rx) = mpsc::sync_channel(1);
        let task2 = scope
            .submit(descriptor(2, policy), move |_| {
                ran_tx.send(()).expect("owner should observe execution");
            })
            .expect("work without a prerequisite should be admitted");
        ran_rx
            .recv_timeout(WAIT)
            .expect("work without a prerequisite must not wait behind the pending task");
        task2.wait();

        assert!(!scope.wait_until_quiescent(UE_PIPE_WAIT), "{policy:?}");
        assert!(!task1.is_complete(), "{policy:?}");
        assert!(task2.is_complete(), "{policy:?}");

        trigger_tx
            .send(())
            .expect("prerequisite gate should still be live");

        assert!(scope.wait_until_quiescent(WAIT), "{policy:?}");
        assert_eq!(task1.status().state, TaskState::Completed, "{policy:?}");
        assert_eq!(task2.status().state, TaskState::Completed, "{policy:?}");
        let report = runtime
            .shutdown(WAIT)
            .expect("drained graph should shut down");
        assert!(!report.has_in_flight_work());
        assert_eq!(prerequisite.status().state, TaskState::Completed);
        assert_eq!(scope.census().completed, 2, "{policy:?}");
        assert_eq!(scope.census().cancelled, 0, "{policy:?}");
    }
}
