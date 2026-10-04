use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;

use super::{
    JobSchedulerReport, TaskPoolKind, TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadCounts,
    JOB_SCHEDULER_DIAGNOSTIC_CAPACITY,
};

const BENCH_SAMPLE_COUNT: usize = 11;
const BENCH_ITERATIONS: usize = 20_000;

#[test]
fn job_scheduler_diagnostics_single_buffer_matches_legacy_lines() {
    let report = representative_report();
    let expected = report.diagnostic_lines().join("\n");
    let mut output = String::with_capacity(JOB_SCHEDULER_DIAGNOSTIC_CAPACITY);
    let initial_capacity = output.capacity();

    report.write_diagnostics(&mut output);

    assert_eq!(output, expected);
    assert_eq!(output.capacity(), initial_capacity);
    assert_eq!(report.format_diagnostics(), expected);
}

#[test]
#[ignore = "managed release benchmark"]
fn job_scheduler_diagnostics_single_buffer_benchmark() {
    let report = representative_report();
    let mut retired_samples_ns = Vec::with_capacity(BENCH_SAMPLE_COUNT);
    let mut optimized_samples_ns = Vec::with_capacity(BENCH_SAMPLE_COUNT);

    for sample in 0..BENCH_SAMPLE_COUNT {
        if sample % 2 == 0 {
            retired_samples_ns.push(measure_retired(&report));
            optimized_samples_ns.push(measure_optimized(&report));
        } else {
            optimized_samples_ns.push(measure_optimized(&report));
            retired_samples_ns.push(measure_retired(&report));
        }
    }

    retired_samples_ns.sort_unstable();
    optimized_samples_ns.sort_unstable();
    let p95_index = (BENCH_SAMPLE_COUNT * 95).div_ceil(100) - 1;
    let retired_p95_ns = retired_samples_ns[p95_index];
    let optimized_p95_ns = optimized_samples_ns[p95_index];
    let reduction_percent = 100.0 * (1.0 - optimized_p95_ns as f64 / retired_p95_ns.max(1) as f64);
    eprintln!(
        "TASK_DIAGNOSTICS_SINGLE_BUFFER samples={BENCH_SAMPLE_COUNT} iterations={BENCH_ITERATIONS} retired_structural_allocations_per_format=15 optimized_structural_allocations_per_format=1 retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_percent={reduction_percent:.3}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(60),
        "single-buffer formatting P95 must be at most 60% of retired line materialization"
    );
}

#[test]
fn optimization_batch_hi_runtime592_task_pool_single_buffer_matches_legacy_lines() {
    let report = representative_pool_report();
    let expected = report.diagnostic_lines().join("\n");

    assert_eq!(report.format_diagnostics(), expected);

    let source = include_str!("../report.rs");
    let format_body = source
        .split("impl TaskPoolReport")
        .nth(1)
        .and_then(|body| body.split("impl TaskPoolReportEntry").next())
        .expect("task pool report implementation");
    assert!(!format_body.contains("self.diagnostic_lines().join"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_runtime592_task_pool_single_buffer_benchmark() {
    const SAMPLES: usize = 17;
    const ITERATIONS: usize = 20_000;
    let report = representative_pool_report();
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(report.diagnostic_lines().join("\n"));
            }
            started.elapsed().as_nanos().max(1)
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(report.format_diagnostics());
            }
            started.elapsed().as_nanos().max(1)
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME592_TASK_POOL_REPORT_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLES} iterations={ITERATIONS} pools=3 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}"
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[test]
fn optimization_batch_hs_runtime601_static_task_metrics_preserve_snapshot() {
    let report = representative_report();
    let mut legacy = DiagnosticStore::new(2);
    let mut optimized = DiagnosticStore::new(2);

    record_task_diagnostics_legacy(&report, &mut legacy, 11);
    report.record_diagnostics(&mut optimized, 11);

    assert_eq!(optimized.snapshot(), legacy.snapshot());
}

#[test]
fn optimization_batch_hs_runtime601_task_report_uses_static_diagnostics() {
    let source = include_str!("../report.rs");
    let record_body = source
        .split("pub fn record_diagnostics(&self, store: &mut DiagnosticStore")
        .nth(1)
        .expect("job scheduler diagnostic projection")
        .split("#[derive(Clone, Debug, PartialEq, Eq)]")
        .next()
        .expect("job scheduler diagnostic body");

    assert!(record_body.contains("store.record_static("));
    assert!(!record_body.contains("store.record("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hs_runtime601_static_task_metrics_p95() {
    const MARKER: &str = "RUNTIME601_TASK_REPORT_STATIC_DIAGNOSTICS_BENCH_V1";
    const SAMPLES: usize = 17;
    const ITERATIONS: usize = 4_096;
    let report = representative_report();
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut optimized = Vec::with_capacity(SAMPLES);
    for pair in 0..SAMPLES {
        if pair % 2 == 0 {
            legacy.push(measure_task_metrics(&report, ITERATIONS, false));
            optimized.push(measure_task_metrics(&report, ITERATIONS, true));
        } else {
            optimized.push(measure_task_metrics(&report, ITERATIONS, true));
            legacy.push(measure_task_metrics(&report, ITERATIONS, false));
        }
    }

    let legacy_p95_ns = percentile_duration_ns(&legacy, 95);
    let optimized_p95_ns = percentile_duration_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} samples={SAMPLES} iterations={ITERATIONS} metrics=13"
    );
    assert!(
        ratio <= 0.50,
        "{MARKER} expected static diagnostic ratio <= 0.50, got {ratio:.4}"
    );
}

fn representative_pool_report() -> TaskPoolReport {
    TaskPoolReport {
        thread_counts: TaskPoolThreadCounts {
            total_threads: 12,
            io_threads: 2,
            async_compute_threads: 2,
            compute_threads: 8,
        },
        pools: vec![
            TaskPoolReportEntry {
                kind: TaskPoolKind::Io,
                thread_name: "zircon-io-task".to_string(),
                configured_worker_threads: Some(2),
                parallelism: 2,
            },
            TaskPoolReportEntry {
                kind: TaskPoolKind::AsyncCompute,
                thread_name: "zircon-async-compute-task".to_string(),
                configured_worker_threads: Some(2),
                parallelism: 2,
            },
            TaskPoolReportEntry {
                kind: TaskPoolKind::Compute,
                thread_name: "zircon-compute-task".to_string(),
                configured_worker_threads: None,
                parallelism: 8,
            },
        ],
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn record_task_diagnostics_legacy(
    report: &JobSchedulerReport,
    store: &mut DiagnosticStore,
    frame_index: u64,
) {
    for (path, value, unit) in [
        (
            super::TASKS_SCHEDULED_DIAGNOSTIC,
            report.scheduled as f64,
            Some("task"),
        ),
        (
            super::TASKS_COMPLETED_DIAGNOSTIC,
            report.completed as f64,
            Some("task"),
        ),
        (
            super::TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
            report.dependency_waiting as f64,
            Some("task"),
        ),
        (
            super::TASKS_QUEUED_DIAGNOSTIC,
            report.queued as f64,
            Some("task"),
        ),
        (
            super::TASKS_ACTIVE_DIAGNOSTIC,
            report.active as f64,
            Some("task"),
        ),
        (
            super::TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC,
            report.queue_wait_samples as f64,
            Some("sample"),
        ),
        (
            super::TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
            report.queue_wait_ms,
            Some("ms"),
        ),
        (
            super::TASKS_EXECUTION_SAMPLES_DIAGNOSTIC,
            report.execution_samples as f64,
            Some("sample"),
        ),
        (
            super::TASKS_EXECUTION_MS_DIAGNOSTIC,
            report.execution_ms,
            Some("ms"),
        ),
        (
            super::TASKS_PANICKED_DIAGNOSTIC,
            report.panicked as f64,
            Some("task"),
        ),
        (
            super::TASKS_CANCELLED_DIAGNOSTIC,
            report.cancelled as f64,
            Some("task"),
        ),
        (
            super::TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC,
            report.dependency_wait_ms,
            Some("ms"),
        ),
        (
            super::TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
            report.explicit_wait_ms,
            Some("ms"),
        ),
    ] {
        store.record(path, frame_index, value, unit, ["tasks", "job_scheduler"]);
    }
}

fn measure_task_metrics(
    report: &JobSchedulerReport,
    iterations: usize,
    optimized: bool,
) -> Duration {
    let mut store = DiagnosticStore::new(1);
    let started = Instant::now();
    for frame_index in 0..iterations as u64 {
        if optimized {
            report.record_diagnostics(&mut store, frame_index);
        } else {
            record_task_diagnostics_legacy(report, &mut store, frame_index);
        }
    }
    let elapsed = started.elapsed();
    black_box(store.current_snapshot());
    elapsed
}

fn percentile_duration_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut ordered = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    ordered.sort_unstable();
    ordered[(ordered.len() - 1) * percentile / 100]
}

fn measure_retired(report: &JobSchedulerReport) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        black_box(report.diagnostic_lines().join("\n"));
    }
    started.elapsed().as_nanos()
}

fn measure_optimized(report: &JobSchedulerReport) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        black_box(report.format_diagnostics());
    }
    started.elapsed().as_nanos()
}

fn representative_report() -> JobSchedulerReport {
    JobSchedulerReport {
        scheduled: 12_345,
        completed: 12_000,
        dependency_waiting: 45,
        queued: 200,
        active: 100,
        queue_wait_samples: 12_100,
        queue_wait_ms: 98.765,
        execution_samples: 12_000,
        execution_ms: 1_234.567,
        panicked: 3,
        cancelled: 7,
        dependency_wait_ms: 45.678,
        explicit_wait_ms: 12.345,
    }
}
