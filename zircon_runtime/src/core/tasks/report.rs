use super::{TaskPool, TaskPoolKind, TaskPoolThreadCounts};

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
