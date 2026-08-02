use crate::core::diagnostics::DiagnosticStore;

use super::{
    TaskPool, TaskPoolKind, TaskPoolThreadCounts, TASKS_ACTIVE_DIAGNOSTIC,
    TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JobSchedulerReport {
    pub scheduled: u64,
    pub completed: u64,
    pub dependency_waiting: u64,
    pub queued: u64,
    pub active: u64,
    pub queue_wait_samples: u64,
    pub queue_wait_ms: f64,
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
        self.diagnostic_lines().join("\n")
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
        self.diagnostic_lines().join("\n")
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
        format!(
            "task_pool.kind={:?} parallelism={} configured_worker_threads={} thread_name={}",
            self.kind,
            self.parallelism,
            self.configured_worker_threads
                .map(|threads| threads.to_string())
                .unwrap_or_else(|| "auto".to_string()),
            self.thread_name
        )
    }
}
