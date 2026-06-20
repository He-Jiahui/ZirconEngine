use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use crossbeam_channel::{bounded, unbounded};

use crate::core::{
    diagnostics::DiagnosticStore, parallel_for, CoreRuntime, JobHandle, JobScheduler, TaskPool,
    TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC,
    TASKS_SCHEDULED_DIAGNOSTIC,
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
        report.main_thread_wait_ms > 0.0,
        "main-thread wait should capture explicit handle wait time"
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
    assert!(diagnostic_current(&snapshot, TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC) > 0.0);
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
    ));
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
        scheduler.diagnostic_report().main_thread_wait_ms > 0.0,
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
