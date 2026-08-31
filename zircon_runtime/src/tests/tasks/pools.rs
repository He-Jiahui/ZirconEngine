use std::time::Duration;

use crossbeam_channel::unbounded;

use crate::core::{CoreRuntime, EngineTaskGraphOptions, TaskPoolKind, TaskPoolOptions};

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
fn core_runtime_owns_one_worker_set_and_keeps_job_scheduler_as_a_facade() {
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("runtime task graph should initialize");
    let worker_pool = runtime.task_graph().worker_pool();
    let inventory = runtime.task_graph_worker_inventory();

    assert_eq!(inventory.worker_set_count, 1);
    assert_eq!(inventory.worker_count, 3);
    assert_eq!(worker_pool.parallelism(), runtime.scheduler().parallelism());
    assert!(runtime.scheduler().shares_execution_owner_with(worker_pool));
    assert_eq!(runtime.scheduler().install(|| 7), 7);
    assert_eq!(worker_pool.join(|| 2, || 5), (2, 5));
}

#[test]
fn isolated_runtimes_own_distinct_task_graph_worker_sets() {
    let first =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(1))
            .expect("first runtime task graph should initialize");
    let second =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(1))
            .expect("second runtime task graph should initialize");

    assert!(!first
        .task_graph()
        .worker_pool()
        .shares_execution_owner_with(second.task_graph().worker_pool()));
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
