use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;
use crate::core::runtime::tasks::{JobHandle, JobScheduler, TaskPool, TaskPoolDescriptor};

fn lane(limits: BoundedKeyedIoLimits) -> BoundedKeyedIoLane {
    BoundedKeyedIoLane::new(
        limits,
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    )
}

fn blocked_scheduler() -> (JobScheduler, mpsc::SyncSender<()>, JobHandle) {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let blocker = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    started_rx.recv().unwrap();
    (scheduler, release_tx, blocker)
}

#[test]
fn preference_persistence_lane_rejects_before_retaining_over_capacity() {
    let lane = lane(BoundedKeyedIoLimits::new(1, 4));
    let admission = lane
        .try_admit(
            "a",
            1,
            4,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    assert_eq!(lane.diagnostics().queue_entries, 1);
    assert!(matches!(
        lane.try_admit(
            "b",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        ),
        Err(BoundedKeyedIoAdmissionError::EntryCapacityExceeded)
    ));
    drop(admission);
    assert_eq!(lane.diagnostics().queue_entries, 0);
}

#[test]
fn preference_persistence_lane_coalesces_same_key_without_crossing_fence() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = Arc::clone(&calls);
    let first = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                first_calls.lock().unwrap().push(1);
                Ok(())
            }),
        )
        .unwrap();
    let second_calls = Arc::clone(&calls);
    let second = lane
        .try_admit(
            "key",
            2,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                second_calls.lock().unwrap().push(2);
                Ok(())
            }),
        )
        .unwrap();
    let first_ticket = first.ticket();
    first.activate();
    let second_ticket = second.activate();
    assert!(matches!(
        first_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Superseded { successor: 2 })
    ));
    release_tx.send(()).unwrap();
    assert!(matches!(
        second_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
    assert_eq!(*calls.lock().unwrap(), vec![2]);
}

#[test]
fn preference_persistence_lane_global_fence_separates_same_key_coalescing_epochs() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let calls = Arc::new(Mutex::new(Vec::new()));

    let first_calls = Arc::clone(&calls);
    let first = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                first_calls.lock().unwrap().push(1);
                Ok(())
            }),
        )
        .unwrap()
        .activate();
    let fence_calls = Arc::clone(&calls);
    let fence = lane
        .submit_fence(
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                fence_calls.lock().unwrap().push(0);
                Ok(())
            }),
        )
        .unwrap();
    let second_calls = Arc::clone(&calls);
    let second = lane
        .try_admit(
            "key",
            2,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                second_calls.lock().unwrap().push(2);
                Ok(())
            }),
        )
        .unwrap()
        .activate();

    release_tx.send(()).unwrap();
    for ticket in [first, fence.ticket().clone(), second] {
        assert!(matches!(
            ticket.wait_until(Instant::now() + Duration::from_secs(2)),
            BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
        ));
    }
    blocker.wait();
    assert_eq!(*calls.lock().unwrap(), vec![1, 0, 2]);
}

#[test]
fn preference_persistence_lane_reverse_activation_keeps_latest_same_key_generation() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = Arc::clone(&calls);
    let first = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                first_calls.lock().unwrap().push(1);
                Ok(())
            }),
        )
        .unwrap();
    let second_calls = Arc::clone(&calls);
    let second = lane
        .try_admit(
            "key",
            2,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                second_calls.lock().unwrap().push(2);
                Ok(())
            }),
        )
        .unwrap();

    let second_ticket = second.activate();
    let first_ticket = first.activate();
    assert!(matches!(
        first_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Superseded { successor: 2 })
    ));
    release_tx.send(()).unwrap();
    assert!(matches!(
        second_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
    assert_eq!(*calls.lock().unwrap(), vec![2]);
}

#[test]
fn preference_persistence_lane_suspended_admission_runs_before_fence() {
    let lane = lane(BoundedKeyedIoLimits::new(8, 1024));
    let calls = Arc::new(Mutex::new(Vec::new()));
    let work_calls = Arc::clone(&calls);
    let admission = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                work_calls.lock().unwrap().push(1);
                Ok(())
            }),
        )
        .unwrap();
    let fence_calls = Arc::clone(&calls);
    let fence = lane
        .submit_fence(
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                fence_calls.lock().unwrap().push(2);
                Ok(())
            }),
        )
        .unwrap();
    assert_eq!(
        fence.ticket().wait_until(Instant::now()),
        BoundedKeyedIoWaitResult::ObserverTimedOut
    );
    let work = admission.activate();
    assert!(matches!(
        work.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    assert!(matches!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    assert_eq!(*calls.lock().unwrap(), vec![1, 2]);
}

#[test]
fn preference_persistence_lane_cancelled_pre_fence_obligation_fails_fence() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let admission = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.activate();
    ticket.cancel_before_start(&authority).unwrap();
    let fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
        .unwrap();
    assert_eq!(
        ticket.terminal(),
        Some(BoundedKeyedIoTerminal::CancelledBeforeStart)
    );
    release_tx.send(()).unwrap();
    assert!(matches!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(BoundedKeyedIoFailure {
            code: "pre_fence_obligation_failed"
        }))
    ));
    blocker.wait();
}

#[test]
fn preference_persistence_lane_fence_pins_pre_epoch_cancel_authority() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let admission = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.activate();
    let fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
        .unwrap();
    assert_eq!(
        ticket.cancel_before_start(&authority),
        Err(BoundedKeyedIoCancelError::FencePinned)
    );
    release_tx.send(()).unwrap();
    assert!(matches!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
}

#[test]
fn preference_persistence_lane_observer_timeout_does_not_cancel_shared_work() {
    let lane = lane(BoundedKeyedIoLimits::new(4, 1024));
    let admission = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let ticket = admission.activate();
    let _ = ticket.wait_until(Instant::now());
    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
}
