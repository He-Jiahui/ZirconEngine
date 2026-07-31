use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier, Mutex,
};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, unbounded};

use crate::core::{
    diagnostics::DiagnosticStore, parallel_for, CoreRuntime, JobHandle, JobScheduler, TaskPool,
    TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TASKS_ACTIVE_DIAGNOSTIC,
    TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
};

#[test]
fn task_pool_options_allocate_bevy_style_default_thread_counts() {
    let counts = TaskPoolOptions::default().resolve_thread_counts(8);

    assert_eq!(counts.total_threads, 8);
    assert_eq!(counts.io_threads, 2);
    assert_eq!(counts.async_compute_threads, 2);
    assert_eq!(counts.compute_threads, 4);
}

#[test]
fn task_pool_options_keep_each_default_pool_available_on_small_hosts() {
    let counts = TaskPoolOptions::default().resolve_thread_counts(2);

    assert_eq!(counts.total_threads, 2);
    assert_eq!(counts.io_threads, 1);
    assert_eq!(counts.async_compute_threads, 1);
    assert_eq!(counts.compute_threads, 1);
}

#[test]
fn task_pools_spawn_work_on_compute_async_compute_and_io_pools() {
    let pools = TaskPoolOptions::with_num_threads(3).create_pools();
    let (sender, receiver) = unbounded();

    for kind in [
        TaskPoolKind::Compute,
        TaskPoolKind::AsyncCompute,
        TaskPoolKind::Io,
    ] {
        let sender = sender.clone();
        pools.get(kind).spawn(move || sender.send(kind).unwrap());
    }
    drop(sender);

    let received = (0..3)
        .map(|_| receiver.recv_timeout(Duration::from_secs(2)).unwrap())
        .collect::<Vec<_>>();

    assert!(received.contains(&TaskPoolKind::Compute));
    assert!(received.contains(&TaskPoolKind::AsyncCompute));
    assert!(received.contains(&TaskPoolKind::Io));
}

#[test]
fn task_pools_report_formats_pool_thread_diagnostics() {
    let pools = TaskPoolOptions::with_num_threads(8).create_pools();
    let report = pools.report();
    let diagnostics = report.format_diagnostics();
    let compute = report
        .entry(TaskPoolKind::Compute)
        .expect("compute pool should be reported");
    let async_compute = report
        .entry(TaskPoolKind::AsyncCompute)
        .expect("async compute pool should be reported");
    let io = report
        .entry(TaskPoolKind::Io)
        .expect("io pool should be reported");

    assert_eq!(report.thread_counts.total_threads, 8);
    assert_eq!(report.pools.len(), 3);
    assert_eq!(compute.parallelism, 4);
    assert_eq!(compute.configured_worker_threads, Some(4));
    assert_eq!(async_compute.parallelism, 2);
    assert_eq!(io.parallelism, 2);

    for expected in [
        "tasks.total_threads=8",
        "tasks.io_threads=2",
        "tasks.async_compute_threads=2",
        "tasks.compute_threads=4",
        "tasks.pools=3",
        "task_pool.kind=Compute",
        "configured_worker_threads=4",
        "thread_name=zircon-compute-task",
    ] {
        assert!(
            diagnostics.contains(expected),
            "task pool diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn core_runtime_exposes_task_pools_and_keeps_job_scheduler_as_compute_facade() {
    let runtime = CoreRuntime::new();
    let compute = runtime.task_pool(TaskPoolKind::Compute);
    let runtime_report = runtime.task_pool_report();
    let handle_report = runtime.handle().task_pool_report();

    assert_eq!(runtime.task_pools().compute().kind(), TaskPoolKind::Compute);
    assert_eq!(compute.parallelism(), runtime.scheduler().parallelism());
    assert_eq!(runtime_report, handle_report);
    assert_eq!(
        runtime_report
            .entry(TaskPoolKind::Compute)
            .expect("runtime report should include compute pool")
            .parallelism,
        runtime.scheduler().parallelism()
    );
    assert_eq!(runtime.scheduler().install(|| 7), 7);
    assert_eq!(compute.join(|| 2, || 5), (2, 5));
}

#[test]
fn isolated_runtime_fixtures_share_the_process_task_owner() {
    const RUNTIME_COUNT: usize = 128;
    let runtimes = (0..RUNTIME_COUNT)
        .map(|_| CoreRuntime::new())
        .collect::<Vec<_>>();
    let first = &runtimes[0];

    for runtime in &runtimes[1..] {
        for kind in [
            TaskPoolKind::Compute,
            TaskPoolKind::AsyncCompute,
            TaskPoolKind::Io,
        ] {
            assert!(
                first
                    .task_pool(kind)
                    .shares_execution_owner_with(runtime.task_pool(kind)),
                "isolated runtime state should reuse the process {kind:?} pool"
            );
        }
    }
}

#[test]
fn explicit_task_pool_options_create_an_isolated_task_owner() {
    let first = TaskPoolOptions::with_num_threads(3).create_pools();
    let second = TaskPoolOptions::with_num_threads(3).create_pools();

    for kind in [
        TaskPoolKind::Compute,
        TaskPoolKind::AsyncCompute,
        TaskPoolKind::Io,
    ] {
        assert!(
            !first
                .get(kind)
                .shares_execution_owner_with(second.get(kind)),
            "explicit pool construction should remain isolated for {kind:?}"
        );
    }
}

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
fn job_terminal_observer_registered_before_completion_runs_once() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (observed_tx, observed_rx) = bounded::<()>(1);
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    let observer_runs_for_callback = Arc::clone(&observer_runs);
    handle.on_terminal(move || {
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
        observed_tx.send(()).unwrap();
    });

    assert_eq!(observer_runs.load(Ordering::SeqCst), 0);
    release_tx.send(()).unwrap();
    observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn job_terminal_observer_registered_after_completion_runs_once() {
    let scheduler = single_worker_scheduler();
    let handle = scheduler.schedule(|| {});
    handle.wait();
    let observer_runs = Arc::new(AtomicUsize::new(0));

    let observer_runs_for_callback = Arc::clone(&observer_runs);
    handle.on_terminal(move || {
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(observer_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn multiple_job_terminal_observers_each_run_exactly_once() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (observed_tx, observed_rx) = bounded::<()>(3);
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    for _ in 0..3 {
        let observer_runs_for_callback = Arc::clone(&observer_runs);
        let observed_tx = observed_tx.clone();
        handle.on_terminal(move || {
            observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
            observed_tx.send(()).unwrap();
        });
    }

    release_tx.send(()).unwrap();
    for _ in 0..3 {
        observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    handle.wait();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 3);
}

#[test]
fn job_terminal_observer_panic_is_contained_and_recorded() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let (survivor_tx, survivor_rx) = bounded::<()>(1);
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());

    handle.on_terminal(|| panic!("terminal observer failure"));
    handle.on_terminal(move || survivor_tx.send(()).unwrap());
    release_tx.send(()).unwrap();
    survivor_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(handle.terminal_observer_panic_count(), 1);
    handle.on_terminal(|| panic!("late terminal observer failure"));
    assert_eq!(handle.terminal_observer_panic_count(), 2);
    handle.wait();
}

#[test]
fn job_terminal_observer_preserves_dependency_continuation_order() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ));
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let (dependent_tx, dependent_rx) = bounded::<()>(1);
    let dependent = scheduler.schedule_after(&[dependency.clone()], move || {
        dependent_tx.send(()).unwrap();
    });
    let (observer_tx, observer_rx) = bounded::<()>(1);

    dependency.on_terminal(move || {
        dependent_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("dependency continuation must launch before terminal observers run");
        observer_tx.send(()).unwrap();
    });

    release_tx.send(()).unwrap();
    observer_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    dependency.wait();
    dependent.wait();

    assert_eq!(dependency.terminal_observer_panic_count(), 0);
}

#[test]
fn job_terminal_observer_can_reenter_handle_accessors() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let handle =
        scheduler.schedule(move || release_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let handle_for_callback = handle.clone();
    let observer_runs = Arc::new(AtomicUsize::new(0));
    let observer_runs_for_callback = Arc::clone(&observer_runs);
    let (observed_tx, observed_rx) = bounded::<()>(1);

    handle.on_terminal(move || {
        assert!(handle_for_callback.is_complete());
        observer_runs_for_callback.fetch_add(1, Ordering::SeqCst);
        let observer_runs_for_nested = Arc::clone(&observer_runs_for_callback);
        handle_for_callback.on_terminal(move || {
            observer_runs_for_nested.fetch_add(1, Ordering::SeqCst);
        });
        observed_tx.send(()).unwrap();
    });

    release_tx.send(()).unwrap();
    observed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.wait();

    assert_eq!(observer_runs.load(Ordering::SeqCst), 2);
    assert_eq!(handle.terminal_observer_panic_count(), 0);
}

#[test]
fn job_diagnostics_track_schedule_complete_and_wait_times() {
    let scheduler = single_worker_scheduler();
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    let after = scheduler.schedule_after(&[dependency], || {});
    let release_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        release_tx.send(()).unwrap();
    });

    after.wait();
    release_thread.join().unwrap();

    let report = scheduler.diagnostic_report();
    assert_eq!(report.scheduled, 2);
    assert_eq!(report.completed, 2);
    assert!(
        report.dependency_wait_ms > 0.0,
        "dependency wait should capture time spent waiting for prerequisites"
    );
    assert!(
        report.explicit_wait_ms > 0.0,
        "explicit wait should capture handle synchronization time"
    );
    let formatted = report.format_diagnostics();
    assert!(formatted.contains("tasks.scheduled=2"));
    assert!(formatted.contains("tasks.completed=2"));

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 7);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, TASKS_SCHEDULED_DIAGNOSTIC),
        2.0
    );
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_COMPLETED_DIAGNOSTIC),
        2.0
    );
    assert!(diagnostic_current(&snapshot, TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC) > 0.0);
    assert!(diagnostic_current(&snapshot, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC) > 0.0);
}

#[test]
fn task_diagnostics_track_ready_queue_active_and_queue_wait() {
    let scheduler = single_worker_scheduler();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let first = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    let second = scheduler.schedule(|| {});
    let saturated = scheduler.diagnostic_report();
    assert_eq!(saturated.scheduled, 2);
    assert_eq!(saturated.queued, 1);
    assert_eq!(saturated.active, 1);
    assert_eq!(saturated.completed, 0);

    release_tx.send(()).unwrap();
    scheduler.wait_all(&[first, second]);

    let drained = scheduler.diagnostic_report();
    assert_eq!(drained.queued, 0);
    assert_eq!(drained.active, 0);
    assert_eq!(drained.completed, 2);
    assert_eq!(drained.queue_wait_samples, 2);
    assert!(drained.queue_wait_ms > 0.0);

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 9);
    let snapshot = store.snapshot();
    assert_eq!(diagnostic_current(&snapshot, TASKS_QUEUED_DIAGNOSTIC), 0.0);
    assert_eq!(diagnostic_current(&snapshot, TASKS_ACTIVE_DIAGNOSTIC), 0.0);
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC),
        2.0
    );
    assert!(diagnostic_current(&snapshot, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC) > 0.0);
}

#[test]
fn task_diagnostics_track_dependency_waiting_through_release_and_cancellation() {
    let scheduler = single_worker_scheduler();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let dependent = scheduler.schedule_after(&[dependency.clone()], || {});

    let waiting = scheduler.diagnostic_report();
    assert_eq!(waiting.scheduled, 2);
    assert_eq!(waiting.completed, 0);
    assert_eq!(waiting.dependency_waiting, 1);
    assert_eq!(waiting.queued, 0);
    assert_eq!(waiting.active, 1);
    assert_eq!(
        waiting.scheduled,
        waiting.completed + waiting.dependency_waiting + waiting.queued + waiting.active
    );

    release_tx.send(()).unwrap();
    scheduler.wait_all(&[dependency, dependent]);
    let released = scheduler.diagnostic_report();
    assert_eq!(released.dependency_waiting, 0);
    assert_eq!(released.completed, 2);

    let cancelled_scheduler = single_worker_scheduler();
    let failed = cancelled_scheduler.schedule(|| panic!("dependency failure"));
    let cancelled = cancelled_scheduler.schedule_after(&[failed], || {
        panic!("cancelled dependent must not run");
    });
    assert!(catch_unwind(AssertUnwindSafe(|| cancelled.wait())).is_err());
    let cancelled_report = cancelled_scheduler.diagnostic_report();
    assert_eq!(cancelled_report.dependency_waiting, 0);
    assert_eq!(cancelled_report.cancelled, 1);
    assert_eq!(
        cancelled_report.queue_wait_samples + cancelled_report.cancelled,
        cancelled_report.completed + cancelled_report.active
    );
    assert_eq!(
        cancelled_report.scheduled,
        cancelled_report.completed
            + cancelled_report.dependency_waiting
            + cancelled_report.queued
            + cancelled_report.active
    );

    let mut store = DiagnosticStore::default();
    cancelled_scheduler.record_diagnostics(&mut store, 10);
    assert_eq!(
        diagnostic_current(&store.snapshot(), TASKS_DEPENDENCY_WAITING_DIAGNOSTIC),
        0.0
    );
}

#[test]
fn task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks() {
    for worker_count in [1, 2, 4] {
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(worker_count),
        ))
        .with_diagnostics();
        let (started_tx, started_rx) = bounded::<()>(worker_count);
        let (release_tx, release_rx) = unbounded::<()>();
        let mut handles = Vec::with_capacity(worker_count * 3);

        for _ in 0..worker_count {
            let started_tx = started_tx.clone();
            let release_rx = release_rx.clone();
            handles.push(scheduler.schedule(move || {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            }));
        }
        for _ in 0..worker_count {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }

        handles.extend((0..worker_count * 2).map(|_| scheduler.schedule(|| {})));
        let saturated = scheduler.diagnostic_report();
        assert_eq!(saturated.active, worker_count as u64);
        assert_eq!(saturated.queued, (worker_count * 2) as u64);
        assert_eq!(saturated.queue_wait_samples, worker_count as u64);

        for _ in 0..worker_count {
            release_tx.send(()).unwrap();
        }
        scheduler.wait_all(&handles);

        let drained = scheduler.diagnostic_report();
        assert_eq!(drained.scheduled, (worker_count * 3) as u64);
        assert_eq!(drained.completed, (worker_count * 3) as u64);
        assert_eq!(drained.queue_wait_samples, (worker_count * 3) as u64);
        assert_eq!(drained.queued, 0);
        assert_eq!(drained.active, 0);
        assert!(drained.queue_wait_ms > 0.0);
    }
}

#[test]
fn task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions() {
    const TASK_COUNT: usize = 128;
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(4),
    ))
    .with_diagnostics();
    let (release_tx, release_rx) = unbounded::<()>();
    let handles = (0..TASK_COUNT)
        .map(|_| {
            let release_rx = release_rx.clone();
            scheduler.schedule(move || {
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            })
        })
        .collect::<Vec<_>>();
    let releaser = std::thread::spawn(move || {
        for _ in 0..TASK_COUNT {
            release_tx.send(()).unwrap();
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_intermediate_completion = false;
    loop {
        let report_started_at = Instant::now();
        let report = scheduler.diagnostic_report();
        assert!(
            report_started_at.elapsed() < Duration::from_millis(250),
            "diagnostic_report must make bounded progress while workers transition"
        );
        assert_eq!(
            report.scheduled,
            report.completed + report.dependency_waiting + report.queued + report.active,
            "a stable lifecycle snapshot must not lose or double-count admitted tasks"
        );
        assert_eq!(
            report.queue_wait_samples + report.cancelled,
            report.completed + report.active,
            "started samples plus never-started cancellations must conserve terminal work"
        );
        saw_intermediate_completion |= report.completed > 0 && report.completed < TASK_COUNT as u64;
        if report.completed == TASK_COUNT as u64 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "diagnostic reports did not advance to the terminal snapshot"
        );
        std::thread::yield_now();
    }

    releaser.join().unwrap();
    scheduler.wait_all(&handles);
    assert!(
        saw_intermediate_completion,
        "reporting must expose at least one stable in-flight lifecycle snapshot"
    );
}

#[test]
fn task_diagnostics_keep_conserved_snapshots_during_concurrent_admission() {
    const PRODUCER_COUNT: usize = 4;
    const TASKS_PER_PRODUCER: usize = 128;
    const TASK_COUNT: usize = PRODUCER_COUNT * TASKS_PER_PRODUCER;

    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(PRODUCER_COUNT),
    ))
    .with_diagnostics();
    let (release_tx, release_rx) = unbounded::<()>();
    let (admission_midpoint_tx, admission_midpoint_rx) = bounded::<()>(PRODUCER_COUNT);
    let (resume_admission_tx, resume_admission_rx) = bounded::<()>(PRODUCER_COUNT);
    let admission_start = Arc::new(Barrier::new(PRODUCER_COUNT + 1));
    let producers_finished = Arc::new(AtomicUsize::new(0));
    let producers = (0..PRODUCER_COUNT)
        .map(|_| {
            let scheduler = scheduler.clone();
            let release_rx = release_rx.clone();
            let admission_midpoint_tx = admission_midpoint_tx.clone();
            let resume_admission_rx = resume_admission_rx.clone();
            let admission_start = Arc::clone(&admission_start);
            let producers_finished = Arc::clone(&producers_finished);
            std::thread::spawn(move || {
                admission_start.wait();
                let handles = (0..TASKS_PER_PRODUCER)
                    .map(|index| {
                        let release_rx = release_rx.clone();
                        let handle = scheduler.schedule(move || {
                            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                        });
                        if index + 1 == TASKS_PER_PRODUCER / 2 {
                            admission_midpoint_tx.send(()).unwrap();
                            resume_admission_rx
                                .recv_timeout(Duration::from_secs(2))
                                .unwrap();
                        }
                        std::thread::yield_now();
                        handle
                    })
                    .collect::<Vec<_>>();
                producers_finished.fetch_add(1, Ordering::Release);
                handles
            })
        })
        .collect::<Vec<_>>();

    admission_start.wait();
    for _ in 0..PRODUCER_COUNT {
        admission_midpoint_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let midpoint_scheduled = (PRODUCER_COUNT * (TASKS_PER_PRODUCER / 2)) as u64;
    let report = loop {
        let report = scheduler.diagnostic_report();
        if report.scheduled == midpoint_scheduled {
            break report;
        }
        assert!(
            Instant::now() < deadline,
            "midpoint reporting did not observe every admitted task"
        );
        std::thread::yield_now();
    };
    assert_eq!(report.scheduled, midpoint_scheduled);
    assert_eq!(report.completed, 0);
    assert_eq!(
        report.scheduled,
        report.completed + report.dependency_waiting + report.queued + report.active,
        "an aggregate snapshot must not span concurrent admission and worker transitions"
    );
    assert_eq!(
        report.queue_wait_samples + report.cancelled,
        report.completed + report.active,
        "queue samples and terminal work must remain conserved while producers submit"
    );
    assert!(
        producers_finished.load(Ordering::Acquire) < PRODUCER_COUNT,
        "the regression must sample before all concurrent producers finish admission"
    );
    for _ in 0..PRODUCER_COUNT {
        resume_admission_tx.send(()).unwrap();
    }

    let mut reports = 1;
    loop {
        let report = scheduler.diagnostic_report();
        assert_eq!(
            report.scheduled,
            report.completed + report.dependency_waiting + report.queued + report.active,
            "an aggregate snapshot must not span concurrent admission and worker transitions"
        );
        assert_eq!(
            report.queue_wait_samples + report.cancelled,
            report.completed + report.active,
            "queue samples and terminal work must remain conserved while producers submit"
        );
        reports += 1;
        if producers_finished.load(Ordering::Acquire) == PRODUCER_COUNT {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "concurrent producers did not finish their bounded admission phase"
        );
        std::thread::yield_now();
    }
    assert!(
        reports > 0,
        "reporting must sample the concurrent admission phase"
    );

    let handles = producers
        .into_iter()
        .flat_map(|producer| producer.join().unwrap())
        .collect::<Vec<_>>();
    for _ in 0..TASK_COUNT {
        release_tx.send(()).unwrap();
    }
    scheduler.wait_all(&handles);

    let terminal = scheduler.diagnostic_report();
    assert_eq!(terminal.scheduled, TASK_COUNT as u64);
    assert_eq!(terminal.completed, TASK_COUNT as u64);
    assert_eq!(terminal.queued, 0);
    assert_eq!(terminal.active, 0);
}

#[test]
fn worker_side_wait_is_reported_as_explicit_wait() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(2),
    ))
    .with_diagnostics();
    let (started_tx, started_rx) = bounded::<()>(1);
    let (release_tx, release_rx) = bounded::<()>(0);
    let dependency = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    let waiter = scheduler.schedule(move || dependency.wait());
    std::thread::sleep(Duration::from_millis(20));
    release_tx.send(()).unwrap();

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !waiter.is_complete() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(waiter.is_complete());

    let report = scheduler.diagnostic_report();
    assert!(report.explicit_wait_ms > 0.0);
    assert!(!report.format_diagnostics().contains("main_thread_wait"));
}

#[test]
fn task_diagnostics_distinguish_panics_from_dependency_cancellation() {
    let scheduler = single_worker_scheduler();
    let dependency = scheduler.schedule(|| panic!("dependency failure"));
    let dependent = scheduler.schedule_after(&[dependency], || {
        panic!("dependency cancellation must prevent this task from running");
    });

    let result = catch_unwind(AssertUnwindSafe(|| dependent.wait()));
    assert!(result.is_err());

    let report = scheduler.diagnostic_report();
    assert_eq!(report.scheduled, 2);
    assert_eq!(report.completed, 2);
    assert_eq!(report.panicked, 1);
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.queued, 0);
    assert_eq!(report.active, 0);

    let mut store = DiagnosticStore::default();
    scheduler.record_diagnostics(&mut store, 11);
    let snapshot = store.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_PANICKED_DIAGNOSTIC),
        1.0
    );
    assert_eq!(
        diagnostic_current(&snapshot, TASKS_CANCELLED_DIAGNOSTIC),
        1.0
    );
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
