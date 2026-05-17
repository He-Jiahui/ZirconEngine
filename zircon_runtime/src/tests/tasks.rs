use std::time::Duration;

use crossbeam_channel::unbounded;

use crate::core::{CoreRuntime, TaskPoolKind, TaskPoolOptions};

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
