use crate::core::{TaskPoolOptions, TaskPoolThreadAssignmentPolicy, TaskPools};

#[test]
fn runtime02_task_pool_options_allocate_bevy_style_default_thread_counts() {
    let counts = TaskPoolOptions::default().resolve_thread_counts(8);

    assert_eq!(counts.total_threads, 8);
    assert_eq!(counts.io_threads, 2);
    assert_eq!(counts.async_compute_threads, 2);
    assert_eq!(counts.compute_threads, 4);
}

#[test]
fn runtime02_task_pool_options_report_the_actual_minimum_worker_budget_on_small_hosts() {
    let counts = TaskPoolOptions::default().resolve_thread_counts(2);

    assert_eq!(counts.total_threads, 3);
    assert_eq!(counts.io_threads, 1);
    assert_eq!(counts.async_compute_threads, 1);
    assert_eq!(counts.compute_threads, 1);
    assert_eq!(
        counts.total_threads,
        counts.io_threads + counts.async_compute_threads + counts.compute_threads
    );
}

#[test]
fn runtime02_task_pool_assignment_conserves_the_reported_worker_budget() {
    for available in [1, 2, 3, 4, 8, 16, 64] {
        let counts = TaskPoolOptions::default().resolve_thread_counts(available);
        assert_eq!(
            counts.total_threads,
            counts.io_threads + counts.async_compute_threads + counts.compute_threads,
            "default assignment must conserve workers for available={available}"
        );
    }

    let mut sparse_percentages = TaskPoolOptions::with_num_threads(7);
    sparse_percentages.io = TaskPoolThreadAssignmentPolicy {
        min_threads: 1,
        max_threads: 4,
        percent: 0.0,
    };
    sparse_percentages.async_compute = TaskPoolThreadAssignmentPolicy {
        min_threads: 1,
        max_threads: 4,
        percent: 0.0,
    };
    sparse_percentages.compute = TaskPoolThreadAssignmentPolicy {
        min_threads: 1,
        max_threads: usize::MAX,
        percent: 0.0,
    };
    let counts = sparse_percentages.resolve_thread_counts(64);
    assert_eq!(counts.total_threads, 7);
    assert_eq!(
        counts.total_threads,
        counts.io_threads + counts.async_compute_threads + counts.compute_threads
    );

    let policy = TaskPoolThreadAssignmentPolicy {
        min_threads: 1,
        max_threads: 4,
        percent: 1.0,
    };
    assert_eq!(policy.thread_count(0, 8), 0);
}

#[test]
fn runtime02_small_host_report_matches_the_created_worker_count() {
    let pools = TaskPools::from_options_with_available_parallelism(TaskPoolOptions::default(), 2);
    let report = pools.report();
    let created_workers = report
        .pools
        .iter()
        .map(|pool| pool.parallelism)
        .sum::<usize>();

    assert_eq!(report.thread_counts.total_threads, 3);
    assert_eq!(created_workers, report.thread_counts.total_threads);
}

#[test]
#[ignore = "managed Runtime02 performance evidence"]
fn runtime02_task_pool_worker_budget_conservation_evidence() {
    for available in [1, 2, 4, 8, 16, 32] {
        let counts = TaskPoolOptions::default().resolve_thread_counts(available);
        let actual_workers =
            counts.io_threads + counts.async_compute_threads + counts.compute_threads;
        assert_eq!(counts.total_threads, actual_workers);
        let legacy_reported_threads = available.max(1);
        let legacy_reporting_error_percent = if actual_workers == legacy_reported_threads {
            0.0
        } else {
            (actual_workers.abs_diff(legacy_reported_threads) as f64
                / legacy_reported_threads as f64)
                * 100.0
        };
        println!(
            "TASK_POOL_BENCH_V1 kind=worker_budget available={} legacy_reported_threads={} legacy_actual_workers={} legacy_reporting_error_percent={:.4} reported_threads_after={} actual_workers_after={} reporting_error_percent_after=0.0000",
            available,
            legacy_reported_threads,
            if available < 3 { 3 } else { available },
            legacy_reporting_error_percent,
            counts.total_threads,
            actual_workers,
        );
    }
}
