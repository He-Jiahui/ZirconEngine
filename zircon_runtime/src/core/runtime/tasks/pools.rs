use std::fmt;

use super::{
    TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TaskPoolReport,
    TaskPoolReportEntry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskPoolThreadCounts {
    pub total_threads: usize,
    pub io_threads: usize,
    pub async_compute_threads: usize,
    pub compute_threads: usize,
}

#[derive(Clone)]
pub struct TaskPools {
    io: TaskPool,
    async_compute: TaskPool,
    compute: TaskPool,
    thread_counts: TaskPoolThreadCounts,
}

impl TaskPools {
    pub fn from_options(options: TaskPoolOptions) -> Self {
        Self::from_options_with_available_parallelism(options, available_parallelism())
    }

    pub fn from_options_with_available_parallelism(
        options: TaskPoolOptions,
        available_parallelism: usize,
    ) -> Self {
        let thread_counts = resolve_thread_counts(&options, available_parallelism);
        Self {
            io: TaskPool::new(
                TaskPoolDescriptor::io().with_worker_threads(thread_counts.io_threads),
            ),
            async_compute: TaskPool::new(
                TaskPoolDescriptor::async_compute()
                    .with_worker_threads(thread_counts.async_compute_threads),
            ),
            compute: TaskPool::new(
                TaskPoolDescriptor::compute().with_worker_threads(thread_counts.compute_threads),
            ),
            thread_counts,
        }
    }

    pub fn thread_counts(&self) -> TaskPoolThreadCounts {
        self.thread_counts
    }

    pub fn get(&self, kind: TaskPoolKind) -> &TaskPool {
        match kind {
            TaskPoolKind::Compute => &self.compute,
            TaskPoolKind::AsyncCompute => &self.async_compute,
            TaskPoolKind::Io => &self.io,
        }
    }

    pub fn compute(&self) -> &TaskPool {
        &self.compute
    }

    pub fn async_compute(&self) -> &TaskPool {
        &self.async_compute
    }

    pub fn io(&self) -> &TaskPool {
        &self.io
    }

    pub fn report(&self) -> TaskPoolReport {
        TaskPoolReport {
            thread_counts: self.thread_counts,
            pools: vec![
                TaskPoolReportEntry::from_pool(&self.io),
                TaskPoolReportEntry::from_pool(&self.async_compute),
                TaskPoolReportEntry::from_pool(&self.compute),
            ],
        }
    }
}

impl Default for TaskPools {
    fn default() -> Self {
        Self::from_options(TaskPoolOptions::default())
    }
}

impl fmt::Debug for TaskPools {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskPools")
            .field("thread_counts", &self.thread_counts)
            .finish()
    }
}

impl TaskPoolOptions {
    pub fn resolve_thread_counts(&self, available_parallelism: usize) -> TaskPoolThreadCounts {
        resolve_thread_counts(self, available_parallelism)
    }

    pub fn create_pools(&self) -> TaskPools {
        TaskPools::from_options(self.clone())
    }
}

fn resolve_thread_counts(
    options: &TaskPoolOptions,
    available_parallelism: usize,
) -> TaskPoolThreadCounts {
    let min_total_threads = options.min_total_threads.max(1);
    let max_total_threads = options.max_total_threads.max(min_total_threads);
    let total_threads = available_parallelism
        .max(1)
        .clamp(min_total_threads, max_total_threads);

    let mut remaining_threads = total_threads;
    let io_threads = options.io.thread_count(remaining_threads, total_threads);
    remaining_threads = remaining_threads.saturating_sub(io_threads);

    let async_compute_threads = options
        .async_compute
        .thread_count(remaining_threads, total_threads);
    remaining_threads = remaining_threads.saturating_sub(async_compute_threads);

    let compute_threads = options
        .compute
        .thread_count(remaining_threads, total_threads);

    TaskPoolThreadCounts {
        total_threads,
        io_threads,
        async_compute_threads,
        compute_threads,
    }
}

fn available_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, |value| value.get())
}
