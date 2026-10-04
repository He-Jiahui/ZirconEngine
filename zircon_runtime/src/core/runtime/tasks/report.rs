use std::fmt::Write as _;

use crate::core::diagnostics::DiagnosticStore;

use super::{
    TaskPool, TaskPoolKind, TaskPoolThreadCounts, TASKS_ACTIVE_DIAGNOSTIC,
    TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXECUTION_MS_DIAGNOSTIC,
    TASKS_EXECUTION_SAMPLES_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
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
            store.record_static(path, frame_index, value, unit, &["tasks", "job_scheduler"]);
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
#[path = "tests/report.rs"]
mod tests;
