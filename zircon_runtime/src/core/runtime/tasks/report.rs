use std::fmt::Write as _;

use crate::core::diagnostics::DiagnosticStore;

use super::{
    TASKS_ACTIVE_DIAGNOSTIC, TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_EXECUTION_MS_DIAGNOSTIC, TASKS_EXECUTION_SAMPLES_DIAGNOSTIC,
    TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC, TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
    TaskPool, TaskPoolKind, TaskPoolThreadCounts,
};

const JOB_SCHEDULER_DIAGNOSTIC_CAPACITY: usize = 512;
const TASK_POOL_DIAGNOSTIC_CAPACITY: usize = 512;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JobSchedulerReport {
    pub scheduled: u64,
    pub completed: u64,
    pub dependency_waiting: u64,
    pub queued: u64,
    pub active: u64,
    pub queue_wait_samples: u64,
    pub queue_wait_ms: f64,
    pub execution_samples: u64,
    pub execution_ms: f64,
    pub panicked: u64,
    pub cancelled: u64,
    pub dependency_wait_ms: f64,
    pub explicit_wait_ms: f64,
}

impl JobSchedulerReport {
    pub fn diagnostic_lines(&self) -> Vec<String> {
        vec![
            format!("{}={}", TASKS_SCHEDULED_DIAGNOSTIC, self.scheduled),
            format!("{}={}", TASKS_COMPLETED_DIAGNOSTIC, self.completed),
            format!(
                "{}={}",
                TASKS_DEPENDENCY_WAITING_DIAGNOSTIC, self.dependency_waiting
            ),
            format!("{}={}", TASKS_QUEUED_DIAGNOSTIC, self.queued),
            format!("{}={}", TASKS_ACTIVE_DIAGNOSTIC, self.active),
            format!(
                "{}={}",
                TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, self.queue_wait_samples
            ),
            format!(
                "{}={:.3}",
                TASKS_QUEUE_WAIT_MS_DIAGNOSTIC, self.queue_wait_ms
            ),
            format!(
                "{}={}",
                TASKS_EXECUTION_SAMPLES_DIAGNOSTIC, self.execution_samples
            ),
            format!("{}={:.3}", TASKS_EXECUTION_MS_DIAGNOSTIC, self.execution_ms),
            format!("{}={}", TASKS_PANICKED_DIAGNOSTIC, self.panicked),
            format!("{}={}", TASKS_CANCELLED_DIAGNOSTIC, self.cancelled),
            format!(
                "{}={:.3}",
                TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, self.dependency_wait_ms
            ),
            format!(
                "{}={:.3}",
                TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC, self.explicit_wait_ms
            ),
        ]
    }

    pub fn format_diagnostics(&self) -> String {
        let mut output = String::with_capacity(JOB_SCHEDULER_DIAGNOSTIC_CAPACITY);
        self.write_diagnostics(&mut output);
        output
    }

    fn write_diagnostics(&self, output: &mut String) {
        write!(
            output,
            "{TASKS_SCHEDULED_DIAGNOSTIC}={}\n\
             {TASKS_COMPLETED_DIAGNOSTIC}={}\n\
             {TASKS_DEPENDENCY_WAITING_DIAGNOSTIC}={}\n\
             {TASKS_QUEUED_DIAGNOSTIC}={}\n\
             {TASKS_ACTIVE_DIAGNOSTIC}={}\n\
             {TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC}={}\n\
             {TASKS_QUEUE_WAIT_MS_DIAGNOSTIC}={:.3}\n\
             {TASKS_EXECUTION_SAMPLES_DIAGNOSTIC}={}\n\
             {TASKS_EXECUTION_MS_DIAGNOSTIC}={:.3}\n\
             {TASKS_PANICKED_DIAGNOSTIC}={}\n\
             {TASKS_CANCELLED_DIAGNOSTIC}={}\n\
             {TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC}={:.3}\n\
             {TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC}={:.3}",
            self.scheduled,
            self.completed,
            self.dependency_waiting,
            self.queued,
            self.active,
            self.queue_wait_samples,
            self.queue_wait_ms,
            self.execution_samples,
            self.execution_ms,
            self.panicked,
            self.cancelled,
            self.dependency_wait_ms,
            self.explicit_wait_ms,
        )
        .expect("writing diagnostics into a String cannot fail");
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        for (path, value, unit) in [
            (
                TASKS_SCHEDULED_DIAGNOSTIC,
                self.scheduled as f64,
                Some("task"),
            ),
            (
                TASKS_COMPLETED_DIAGNOSTIC,
                self.completed as f64,
                Some("task"),
            ),
            (
                TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
                self.dependency_waiting as f64,
                Some("task"),
            ),
            (TASKS_QUEUED_DIAGNOSTIC, self.queued as f64, Some("task")),
            (TASKS_ACTIVE_DIAGNOSTIC, self.active as f64, Some("task")),
            (
                TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC,
                self.queue_wait_samples as f64,
                Some("sample"),
            ),
            (
                TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
                self.queue_wait_ms,
                Some("ms"),
            ),
            (
                TASKS_EXECUTION_SAMPLES_DIAGNOSTIC,
                self.execution_samples as f64,
                Some("sample"),
            ),
            (TASKS_EXECUTION_MS_DIAGNOSTIC, self.execution_ms, Some("ms")),
            (
                TASKS_PANICKED_DIAGNOSTIC,
                self.panicked as f64,
                Some("task"),
            ),
            (
                TASKS_CANCELLED_DIAGNOSTIC,
                self.cancelled as f64,
                Some("task"),
            ),
            (
                TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC,
                self.dependency_wait_ms,
                Some("ms"),
            ),
            (
                TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
                self.explicit_wait_ms,
                Some("ms"),
            ),
        ] {
            store.record(path, frame_index, value, unit, ["tasks", "job_scheduler"]);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskPoolReport {
    pub thread_counts: TaskPoolThreadCounts,
    pub pools: Vec<TaskPoolReportEntry>,
}

impl TaskPoolReport {
    pub fn entry(&self, kind: TaskPoolKind) -> Option<&TaskPoolReportEntry> {
        self.pools.iter().find(|entry| entry.kind == kind)
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.pools.len() + 5);
        lines.push(format!(
            "tasks.total_threads={}",
            self.thread_counts.total_threads
        ));
        lines.push(format!(
            "tasks.io_threads={}",
            self.thread_counts.io_threads
        ));
        lines.push(format!(
            "tasks.async_compute_threads={}",
            self.thread_counts.async_compute_threads
        ));
        lines.push(format!(
            "tasks.compute_threads={}",
            self.thread_counts.compute_threads
        ));
        lines.push(format!("tasks.pools={}", self.pools.len()));
        lines.extend(self.pools.iter().map(TaskPoolReportEntry::diagnostic_line));
        lines
    }

    pub fn format_diagnostics(&self) -> String {
        let mut output = String::with_capacity(TASK_POOL_DIAGNOSTIC_CAPACITY);
        write!(
            output,
            "tasks.total_threads={}\n\
             tasks.io_threads={}\n\
             tasks.async_compute_threads={}\n\
             tasks.compute_threads={}\n\
             tasks.pools={}",
            self.thread_counts.total_threads,
            self.thread_counts.io_threads,
            self.thread_counts.async_compute_threads,
            self.thread_counts.compute_threads,
            self.pools.len(),
        )
        .expect("writing diagnostics into a String cannot fail");
        for entry in &self.pools {
            output.push('\n');
            entry.write_diagnostic_line(&mut output);
        }
        output
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskPoolReportEntry {
    pub kind: TaskPoolKind,
    pub thread_name: String,
    pub configured_worker_threads: Option<usize>,
    pub parallelism: usize,
}

impl TaskPoolReportEntry {
    pub(crate) fn from_pool(pool: &TaskPool) -> Self {
        let descriptor = pool.descriptor();
        Self {
            kind: descriptor.kind,
            thread_name: descriptor.thread_name.clone(),
            configured_worker_threads: descriptor.worker_threads,
            parallelism: pool.parallelism(),
        }
    }

    fn diagnostic_line(&self) -> String {
        let mut output = String::with_capacity(128);
        self.write_diagnostic_line(&mut output);
        output
    }

    fn write_diagnostic_line(&self, output: &mut String) {
        match self.configured_worker_threads {
            Some(threads) => write!(
                output,
                "task_pool.kind={:?} parallelism={} configured_worker_threads={} thread_name={}",
                self.kind, self.parallelism, threads, self.thread_name
            ),
            None => write!(
                output,
                "task_pool.kind={:?} parallelism={} configured_worker_threads=auto thread_name={}",
                self.kind, self.parallelism, self.thread_name
            ),
        }
        .expect("writing diagnostics into a String cannot fail");
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        JOB_SCHEDULER_DIAGNOSTIC_CAPACITY, JobSchedulerReport, TaskPoolKind, TaskPoolReport,
        TaskPoolReportEntry, TaskPoolThreadCounts,
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
        let reduction_percent =
            100.0 * (1.0 - optimized_p95_ns as f64 / retired_p95_ns.max(1) as f64);
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

        let source = include_str!("report.rs");
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
}
