use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier, Mutex,
};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, unbounded};

use crate::core::{
    diagnostics::DiagnosticStore, parallel_for, JobHandle, JobScheduler, TaskPool,
    TaskPoolDescriptor, TASKS_ACTIVE_DIAGNOSTIC, TASKS_CANCELLED_DIAGNOSTIC,
    TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
};

#[path = "tasks/diagnostics.rs"]
mod diagnostics;
#[path = "tasks/pools.rs"]
mod pools;
#[path = "tasks/terminal_observers.rs"]
mod terminal_observers;
#[path = "tasks/thread_budget.rs"]
mod thread_budget;

#[test]
fn job_handle_wait_blocks_until_task_completes() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_for_task = Arc::clone(&completed);

    let handle = scheduler.schedule(move || {
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        completed_for_task.store(1, Ordering::SeqCst);
    });

    assert!(!handle.is_complete());
    release_tx.send(()).unwrap();
    handle.wait();

    assert!(handle.is_complete());
    assert_eq!(completed.load(Ordering::SeqCst), 1);
}

#[test]
fn schedule_after_runs_task_only_after_all_dependencies() {
    let scheduler = single_worker_scheduler();
    let (first_tx, first_rx) = bounded::<()>(0);
    let (second_tx, second_rx) = bounded::<()>(0);
    let events = Arc::new(Mutex::new(Vec::new()));

    let first_events = Arc::clone(&events);
    let first = scheduler.schedule(move || {
        first_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        first_events.lock().unwrap().push("first");
    });
    let second_events = Arc::clone(&events);
    let second = scheduler.schedule(move || {
        second_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        second_events.lock().unwrap().push("second");
    });
    let after_events = Arc::clone(&events);
    let after = scheduler.schedule_after(&[first, second], move || {
        after_events.lock().unwrap().push("after");
    });

    std::thread::sleep(Duration::from_millis(25));
    assert!(!after.is_complete());
    first_tx.send(()).unwrap();
    std::thread::sleep(Duration::from_millis(25));
    assert!(!after.is_complete());
    second_tx.send(()).unwrap();
    after.wait();

    assert_eq!(&*events.lock().unwrap(), &["first", "second", "after"]);
}

#[test]
fn combined_handle_completes_when_all_children_complete() {
    let scheduler = single_worker_scheduler();
    let (first_tx, first_rx) = bounded::<()>(0);
    let (second_tx, second_rx) = bounded::<()>(0);

    let first = scheduler.schedule(move || first_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let second =
        scheduler.schedule(move || second_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let combined = JobHandle::combine(&[first, second]);

    assert!(!combined.is_complete());
    first_tx.send(()).unwrap();
    std::thread::sleep(Duration::from_millis(25));
    assert!(!combined.is_complete());
    second_tx.send(()).unwrap();
    combined.wait();

    assert!(combined.is_complete());
}

#[test]
fn combined_handle_waits_for_all_children_before_propagating_panic() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ));
    let (blocking_started_tx, blocking_started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let panicked = scheduler.schedule(|| panic!("combined child failure"));
    let blocking = scheduler.schedule(move || {
        blocking_started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    blocking_started_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    let combined = JobHandle::combine(&[panicked.clone(), blocking.clone()]);

    let deadline = Instant::now() + Duration::from_secs(2);
    while !panicked.is_complete() {
        assert!(
            Instant::now() < deadline,
            "panicking child did not reach its terminal state"
        );
        std::thread::yield_now();
    }
    assert!(
        !combined.is_complete(),
        "combined handle must retain the barrier until every child is terminal"
    );

    release_tx.send(()).unwrap();
    let wait_result = catch_unwind(AssertUnwindSafe(|| combined.wait()));
    assert!(wait_result.is_err());
    assert!(blocking.is_complete());
    assert!(combined.is_complete());
}

#[test]
fn schedule_after_does_not_consume_worker_while_waiting_on_dependencies() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let events = Arc::new(Mutex::new(Vec::new()));
    let dependency_events = Arc::clone(&events);

    let dependency = scheduler.schedule(move || {
        dependency_events.lock().unwrap().push("dependency-start");
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        dependency_events.lock().unwrap().push("dependency-end");
    });
    let after_events = Arc::clone(&events);
    let after = scheduler.schedule_after(&[dependency], move || {
        after_events.lock().unwrap().push("after");
    });

    std::thread::sleep(Duration::from_millis(25));
    assert!(!after.is_complete());
    release_tx.send(()).unwrap();
    after.wait();

    assert_eq!(
        &*events.lock().unwrap(),
        &["dependency-start", "dependency-end", "after"]
    );
}

#[test]
fn worker_thread_wait_does_not_deadlock_scheduler() {
    let scheduler = single_worker_scheduler();
    let scheduler_for_outer = scheduler.clone();
    let child_ran = Arc::new(AtomicUsize::new(0));
    let child_ran_for_outer = Arc::clone(&child_ran);

    let outer = scheduler.schedule(move || {
        let child_ran_for_child = Arc::clone(&child_ran_for_outer);
        let child = scheduler_for_outer.schedule(move || {
            child_ran_for_child.store(1, Ordering::SeqCst);
        });
        child.wait();
    });

    outer.wait();

    assert!(outer.is_complete());
    assert_eq!(child_ran.load(Ordering::SeqCst), 1);
    assert_eq!(scheduler.diagnostic_report().scheduled, 2);
    assert_eq!(scheduler.diagnostic_report().completed, 2);
}

#[test]
fn task_diagnostics_are_disabled_by_default() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_for_task = Arc::clone(&completed);
    let handle = scheduler.schedule(move || {
        completed_for_task.fetch_add(1, Ordering::SeqCst);
    });

    handle.wait();

    assert_eq!(completed.load(Ordering::SeqCst), 1);
    assert_eq!(scheduler.diagnostic_report(), Default::default());
}

#[test]
fn deep_dependency_chain_completes_in_order() {
    let scheduler = single_worker_scheduler();
    let completed_order = Arc::new(Mutex::new(Vec::new()));
    let mut tail = JobHandle::completed();

    for step in 0..64 {
        let order_for_task = Arc::clone(&completed_order);
        tail = scheduler.schedule_after(&[tail], move || {
            order_for_task.lock().unwrap().push(step);
        });
    }

    tail.wait();

    let expected = (0..64).collect::<Vec<_>>();
    assert_eq!(*completed_order.lock().unwrap(), expected);
    assert!(tail.is_complete());
}

#[test]
fn wide_fanout_combine_waits_for_all() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(4),
    ));
    let completed = Arc::new(AtomicUsize::new(0));
    let handles = (0..128)
        .map(|_| {
            let completed_for_task = Arc::clone(&completed);
            scheduler.schedule(move || {
                completed_for_task.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();
    let combined = JobHandle::combine(&handles);

    combined.wait();

    assert_eq!(completed.load(Ordering::SeqCst), 128);
    assert!(combined.is_complete());
    assert!(handles.iter().all(JobHandle::is_complete));
}

#[test]
fn scheduler_wait_all_waits_for_all_handles_and_records_sync_time() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ))
    .with_diagnostics();
    let (release_tx, release_rx) = bounded::<()>(0);
    let completed = Arc::new(AtomicUsize::new(0));
    let handles = (0..3)
        .map(|_| {
            let release_rx = release_rx.clone();
            let completed_for_task = Arc::clone(&completed);
            scheduler.schedule(move || {
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                completed_for_task.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();
    let release_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        for _ in 0..3 {
            release_tx.send(()).unwrap();
        }
    });

    scheduler.wait_all(&handles);
    release_thread.join().unwrap();

    assert_eq!(completed.load(Ordering::SeqCst), 3);
    assert!(handles.iter().all(JobHandle::is_complete));
    assert!(
        scheduler.diagnostic_report().explicit_wait_ms > 0.0,
        "wait_all should record explicit scheduler synchronization time"
    );
}

#[test]
fn job_handle_wait_reports_task_panic_without_leaking_completion() {
    let scheduler = single_worker_scheduler();

    let handle = scheduler.schedule(|| panic!("scheduled failure"));

    let wait_result = catch_unwind(AssertUnwindSafe(|| handle.wait()));

    assert!(wait_result.is_err());
    assert!(handle.is_complete());
    assert_eq!(scheduler.diagnostic_report().scheduled, 1);
    assert_eq!(scheduler.diagnostic_report().completed, 1);
}

#[test]
fn schedule_after_propagates_dependency_panic_without_running_dependent_task() {
    let scheduler = single_worker_scheduler();
    let dependent_ran = Arc::new(AtomicUsize::new(0));

    let dependency = scheduler.schedule(|| panic!("dependency failure"));
    let dependent_ran_for_task = Arc::clone(&dependent_ran);
    let dependent = scheduler.schedule_after(&[dependency], move || {
        dependent_ran_for_task.fetch_add(1, Ordering::SeqCst);
    });

    let wait_result = catch_unwind(AssertUnwindSafe(|| dependent.wait()));

    assert!(wait_result.is_err());
    assert!(dependent.is_complete());
    assert_eq!(dependent_ran.load(Ordering::SeqCst), 0);
    assert_eq!(scheduler.diagnostic_report().scheduled, 2);
    assert_eq!(scheduler.diagnostic_report().completed, 2);
}

#[test]
fn parallel_for_visits_every_item_exactly_once() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let mut values = vec![0_u32; 128];

    parallel_for(&pool, &mut values, 8, |chunk| {
        for value in chunk {
            *value += 1;
        }
    });

    assert!(values.iter().all(|value| *value == 1));
}

#[test]
fn parallel_for_chunk_size_bounds_task_granularity() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let chunk_lengths = Arc::new(Mutex::new(Vec::new()));
    let lengths_for_task = Arc::clone(&chunk_lengths);
    let mut values = vec![0_u32; 10];

    parallel_for(&pool, &mut values, 4, move |chunk| {
        lengths_for_task.lock().unwrap().push(chunk.len());
        for value in chunk {
            *value = 1;
        }
    });
    let mut lengths = chunk_lengths.lock().unwrap().clone();
    lengths.sort_unstable();

    assert_eq!(lengths, vec![2, 4, 4]);
    assert!(values.iter().all(|value| *value == 1));
}

fn single_worker_scheduler() -> JobScheduler {
    JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ))
    .with_diagnostics()
}

fn diagnostic_current(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> f64 {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
        .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"))
}
