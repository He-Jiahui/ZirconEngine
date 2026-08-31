use std::time::{Duration, Instant};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use super::lane::LaneInner;
use super::*;
use crate::core::runtime::tasks::{JobHandle, JobScheduler, TaskPool, TaskPoolDescriptor};

mod fence_accounting;
mod fence_failures;
mod suspended_order;

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
fn bounded_keyed_io_lane_coalesces_equal_typed_domain_keys() {
    #[derive(Clone, PartialEq, Eq)]
    struct PhysicalPathIdentity(u64);

    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(2, 16), scheduler);
    let first = lane
        .try_admit(
            BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7)),
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let first_ticket = first.ticket();
    first.activate();
    let second_ticket = lane
        .try_admit(
            BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7)),
            2,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();

    assert_eq!(
        first_ticket.terminal(),
        Some(BoundedKeyedIoTerminal::Superseded { successor: 2 })
    );
    release_tx.send(()).unwrap();
    assert_eq!(
        second_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    blocker.wait();
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
    assert_eq!(
        ticket.wait_until(Instant::now()),
        BoundedKeyedIoWaitResult::ObserverTimedOut
    );
    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
}

#[test]
fn preference_persistence_lane_multiple_waiters_share_one_terminal_after_timeout() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(2, 32), scheduler);
    let ticket = lane
        .try_admit(
            "shared",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();
    let second_waiter = ticket.clone();
    assert_eq!(
        ticket.wait_until(Instant::now()),
        BoundedKeyedIoWaitResult::ObserverTimedOut
    );

    release_tx.send(()).unwrap();
    assert_eq!(
        second_waiter.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    blocker.wait();
}

#[test]
fn preference_persistence_lane_deadline_expires_while_io_worker_is_saturated() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(2, 32), scheduler);
    let ticket = lane
        .try_admit(
            "deadline",
            1,
            8,
            BoundedKeyedIoWorkDeadline::at(Instant::now() + Duration::from_millis(10)),
            Box::new(|| panic!("expired work must never acquire the saturated worker")),
        )
        .unwrap()
        .activate();

    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::DeadlineBeforeStart)
    );
    assert_eq!(lane.diagnostics().queue_entries, 0);

    release_tx.send(()).unwrap();
    blocker.wait();
}

#[test]
fn preference_persistence_lane_fence_deadline_expires_while_io_worker_is_saturated() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(2, 32), scheduler);
    let fence = lane
        .submit_fence(
            8,
            BoundedKeyedIoWorkDeadline::at(Instant::now() + Duration::from_millis(10)),
            Box::new(|| panic!("expired fence must never acquire the saturated worker")),
        )
        .unwrap();

    assert_eq!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::DeadlineBeforeStart)
    );
    assert_eq!(lane.diagnostics().queue_entries, 0);

    release_tx.send(()).unwrap();
    blocker.wait();
}

#[test]
fn preference_persistence_lane_expired_fence_releases_cancel_pin() {
    let lane = lane(BoundedKeyedIoLimits::new(4, 64));
    let admission = lane
        .try_admit(
            "pinned-until-expiry",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.ticket();
    let fence = lane
        .submit_fence(
            8,
            BoundedKeyedIoWorkDeadline::at(Instant::now() + Duration::from_millis(10)),
            Box::new(|| Ok(())),
        )
        .unwrap();
    assert_eq!(
        ticket.cancel_before_start(&authority),
        Err(BoundedKeyedIoCancelError::FencePinned)
    );

    assert_eq!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::DeadlineBeforeStart)
    );
    assert_eq!(ticket.cancel_before_start(&authority), Ok(()));
    drop(admission);
}

#[test]
fn preference_persistence_lane_shutdown_drains_fence_pinned_suspended_work() {
    let lane = lane(BoundedKeyedIoLimits::new(4, 64));
    let calls = Arc::new(Mutex::new(Vec::new()));
    let work_calls = Arc::clone(&calls);
    let admission = lane
        .try_admit(
            "pinned",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                work_calls.lock().unwrap().push("work");
                Ok(())
            }),
        )
        .unwrap();
    let ticket = admission.ticket();
    let fence_calls = Arc::clone(&calls);
    let fence = lane
        .submit_fence(
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                fence_calls.lock().unwrap().push("fence");
                Ok(())
            }),
        )
        .unwrap();

    let guard = lane.shutdown();
    assert!(guard.wait_until(Instant::now() + Duration::from_secs(2)));
    assert!(guard.report().complete);
    assert_eq!(guard.report().incomplete_entries, 0);
    assert_eq!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Succeeded));
    assert_eq!(
        fence.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(*calls.lock().unwrap(), vec!["work", "fence"]);

    let activated_after_shutdown = admission.activate();
    assert_eq!(
        activated_after_shutdown.terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
}

#[test]
fn preference_persistence_lane_shutdown_timeout_keeps_queryable_guard_until_work_returns() {
    let lane = lane(BoundedKeyedIoLimits::new(2, 32));
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let ticket = lane
        .try_admit(
            "shutdown-timeout",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            }),
        )
        .unwrap()
        .activate();
    started_rx.recv().unwrap();

    let guard = lane.shutdown();
    assert!(!guard.wait_until(Instant::now() + Duration::from_millis(10)));
    let report = guard.report();
    assert!(!report.complete);
    assert_eq!(report.incomplete_entries, 1);

    release_tx.send(()).unwrap();
    assert!(guard.wait_until(Instant::now() + Duration::from_secs(2)));
    assert_eq!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Succeeded));
}

#[test]
fn preference_persistence_lane_panicking_work_releases_capacity_with_typed_failure() {
    let lane = lane(BoundedKeyedIoLimits::new(1, 8));
    let panicked = lane
        .try_admit(
            "panic",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| panic!("injected backend panic")),
        )
        .unwrap()
        .activate();
    assert_eq!(
        panicked.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(
            BoundedKeyedIoFailure::new("work_panicked")
        ))
    );

    let replacement = lane
        .try_admit(
            "replacement",
            2,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();
    assert!(matches!(
        replacement.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
}

#[test]
fn preference_persistence_lane_repeated_cancel_stays_idempotent_after_fence_pin() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(4, 64), scheduler);
    let admission = lane
        .try_admit(
            "cancelled",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.activate();
    ticket.cancel_before_start(&authority).unwrap();
    let fence = lane
        .submit_fence(8, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
        .unwrap();

    assert_eq!(ticket.cancel_before_start(&authority), Ok(()));
    release_tx.send(()).unwrap();
    assert!(matches!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(_))
    ));
    blocker.wait();
}

#[test]
fn preference_persistence_lane_marks_active_work_started_before_shutdown_can_linearize() {
    let lane = lane(BoundedKeyedIoLimits::new(1, 8));
    let (hook_started_tx, hook_started_rx) = mpsc::sync_channel(0);
    let (release_hook_tx, release_hook_rx) = mpsc::sync_channel(0);
    let release_hook_rx = Arc::new(Mutex::new(release_hook_rx));
    lane.set_before_execute_hook(move || {
        hook_started_tx.send(()).unwrap();
        release_hook_rx.lock().unwrap().recv().unwrap();
    });
    let ran = Arc::new(Mutex::new(false));
    let ran_for_work = Arc::clone(&ran);
    let ticket = lane
        .try_admit(
            "shutdown-linearization",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                *ran_for_work.lock().unwrap() = true;
                Ok(())
            }),
        )
        .unwrap()
        .activate();
    hook_started_rx.recv().unwrap();

    let guard = lane.shutdown();
    assert_eq!(ticket.terminal(), None);
    assert!(!guard.wait_until(Instant::now() + Duration::from_millis(10)));

    release_hook_tx.send(()).unwrap();
    assert!(guard.wait_until(Instant::now() + Duration::from_secs(2)));
    assert_eq!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Succeeded));
    assert!(*ran.lock().unwrap());
}

#[test]
fn preference_persistence_lane_shutdown_waits_for_terminal_observer_and_pump_handle() {
    let lane = lane(BoundedKeyedIoLimits::new(1, 8));
    let (observer_started_tx, observer_started_rx) = mpsc::sync_channel(0);
    let (release_observer_tx, release_observer_rx) = mpsc::sync_channel(0);
    let release_observer_rx = Arc::new(Mutex::new(release_observer_rx));
    let admission = lane
        .try_admit(
            "observer-drain",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    admission.observe_terminal(move |_| {
        observer_started_tx.send(()).unwrap();
        release_observer_rx.lock().unwrap().recv().unwrap();
    });
    let ticket = admission.activate();
    observer_started_rx.recv().unwrap();

    let guard = lane.shutdown();
    assert_eq!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Succeeded));
    assert!(!guard.wait_until(Instant::now() + Duration::from_millis(10)));

    release_observer_tx.send(()).unwrap();
    assert!(guard.wait_until(Instant::now() + Duration::from_secs(2)));
}

#[test]
fn preference_persistence_lane_admission_matrix_stays_bounded_at_one_thousand_and_hundred_thousand()
{
    for scale in [1_usize, 1_000, 100_000] {
        let lane = lane(BoundedKeyedIoLimits::new(scale, scale));
        let admissions = (0..scale)
            .map(|generation| {
                lane.try_admit(
                    "matrix",
                    generation as u64,
                    1,
                    BoundedKeyedIoWorkDeadline::none(),
                    Box::new(|| Ok(())),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(lane.diagnostics().queue_entries, scale);
        assert!(matches!(
            lane.try_admit(
                "over-capacity",
                scale as u64,
                1,
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(|| Ok(())),
            ),
            Err(BoundedKeyedIoAdmissionError::EntryCapacityExceeded)
        ));
        drop(admissions);
        assert_eq!(lane.diagnostics().queue_entries, 0);
    }
}

#[test]
fn preference_persistence_lane_same_key_storm_does_not_starve_an_interleaved_key() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(1_024, 1_024), scheduler);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut superseded = Vec::new();
    for generation in 1..=1_000_u64 {
        let calls_for_work = Arc::clone(&calls);
        let key = if generation == 2 { "other" } else { "storm" };
        let ticket = lane
            .try_admit(
                key,
                generation,
                1,
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(move || {
                    calls_for_work.lock().unwrap().push(generation);
                    Ok(())
                }),
            )
            .unwrap()
            .activate();
        if generation != 2 && generation != 1_000 {
            superseded.push(ticket);
        }
    }
    release_tx.send(()).unwrap();
    blocker.wait();
    let last = superseded
        .last()
        .map(BoundedKeyedIoTicket::generation)
        .unwrap_or_default();
    assert_eq!(last, 999);
    let deadline = Instant::now() + Duration::from_secs(2);
    while lane.diagnostics().queue_entries != 0 {
        assert!(Instant::now() < deadline, "storm did not drain");
        thread::yield_now();
    }
    assert_eq!(*calls.lock().unwrap(), vec![2, 1_000]);
    assert!(superseded.iter().all(|ticket| matches!(
        ticket.terminal(),
        Some(BoundedKeyedIoTerminal::Superseded { .. })
    )));
}

#[test]
fn preference_persistence_lane_active_observer_is_not_lost_or_notified_twice() {
    let lane = lane(BoundedKeyedIoLimits::new(1, 8));
    let (hook_started_tx, hook_started_rx) = mpsc::sync_channel(0);
    let (release_hook_tx, release_hook_rx) = mpsc::sync_channel(0);
    let release_hook_rx = Arc::new(Mutex::new(release_hook_rx));
    lane.set_before_execute_hook(move || {
        hook_started_tx.send(()).unwrap();
        release_hook_rx.lock().unwrap().recv().unwrap();
    });
    let admission = lane
        .try_admit(
            "active-observer",
            1,
            8,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let lane_inner = Arc::clone(&admission.lane);
    let ticket_id = admission.ticket_id;
    let ticket = admission.ticket();
    admission.activate();
    hook_started_rx.recv().unwrap();

    let (observed_tx, observed_rx) = mpsc::channel();
    LaneInner::observe_terminal(
        &lane_inner,
        ticket_id,
        &ticket,
        Arc::new(move |terminal| observed_tx.send(terminal).unwrap()),
    );
    release_hook_tx.send(()).unwrap();

    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        observed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        BoundedKeyedIoTerminal::Succeeded
    );
    assert!(observed_rx.try_recv().is_err());
}
