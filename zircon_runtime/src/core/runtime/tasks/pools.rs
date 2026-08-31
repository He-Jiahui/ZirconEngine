use std::fmt;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::pool::TaskPoolShutdownCensus;
use super::{
    TaskPool, TaskPoolBuildError, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions,
    TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskPoolThreadCounts {
    /// Physical workers created across all pools.
    ///
    /// This can exceed the requested total when the sum of per-pool minimums is larger.
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

static PROCESS_TASK_POOLS: OnceLock<TaskPools> = OnceLock::new();

impl TaskPools {
    /// Returns the process-wide default execution owner shared by default runtimes.
    pub fn process_default() -> Self {
        PROCESS_TASK_POOLS
            .get_or_init(|| Self::from_options(TaskPoolOptions::default()))
            .clone()
    }

    pub fn from_options(options: TaskPoolOptions) -> Self {
        Self::try_from_options(options).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Builds an explicitly owned pool set without retaining a process-global
    /// worker owner. If a later pool cannot start, the already-created pools
    /// are dropped before the construction error is returned.
    pub fn try_from_options(options: TaskPoolOptions) -> Result<Self, TaskPoolBuildError> {
        Self::try_from_options_with_available_parallelism(options, available_parallelism())
    }

    pub fn from_options_with_available_parallelism(
        options: TaskPoolOptions,
        available_parallelism: usize,
    ) -> Self {
        Self::try_from_options_with_available_parallelism(options, available_parallelism)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    pub fn try_from_options_with_available_parallelism(
        options: TaskPoolOptions,
        available_parallelism: usize,
    ) -> Result<Self, TaskPoolBuildError> {
        let thread_counts = resolve_thread_counts(&options, available_parallelism);
        let io = TaskPool::try_new(
            TaskPoolDescriptor::io().with_worker_threads(thread_counts.io_threads),
        )?;
        let async_compute = TaskPool::try_new(
            TaskPoolDescriptor::async_compute()
                .with_worker_threads(thread_counts.async_compute_threads),
        )?;
        let compute = TaskPool::try_new(
            TaskPoolDescriptor::compute().with_worker_threads(thread_counts.compute_threads),
        )?;
        Ok(Self {
            io,
            async_compute,
            compute,
            thread_counts,
        })
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

    pub(super) fn close_and_join(&self, timeout: Duration) -> Vec<TaskPoolShutdownCensus> {
        let started_at = Instant::now();
        let pools = [&self.io, &self.async_compute, &self.compute];
        self.close_admission();
        pools
            .into_iter()
            .map(|pool| pool.close_and_join(timeout.saturating_sub(started_at.elapsed())))
            .collect()
    }

    pub(super) fn close_admission(&self) {
        for pool in [&self.io, &self.async_compute, &self.compute] {
            pool.close_admission();
        }
    }

    pub(super) fn shutdown_census(&self) -> Vec<TaskPoolShutdownCensus> {
        [&self.io, &self.async_compute, &self.compute]
            .into_iter()
            .map(TaskPool::shutdown_census)
            .collect()
    }
}

impl Default for TaskPools {
    fn default() -> Self {
        Self::process_default()
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
    /// Resolves a requested total before enforcing each independent pool's minimum worker count.
    pub fn resolve_thread_counts(&self, available_parallelism: usize) -> TaskPoolThreadCounts {
        resolve_thread_counts(self, available_parallelism)
    }

    pub fn create_pools(&self) -> TaskPools {
        TaskPools::from_options(self.clone())
    }

    pub fn try_create_pools(&self) -> Result<TaskPools, TaskPoolBuildError> {
        TaskPools::try_from_options(self.clone())
    }
}

fn resolve_thread_counts(
    options: &TaskPoolOptions,
    available_parallelism: usize,
) -> TaskPoolThreadCounts {
    let min_total_threads = options.min_total_threads.max(1);
    let max_total_threads = options.max_total_threads.max(min_total_threads);
    let requested_threads = available_parallelism
        .max(1)
        .clamp(min_total_threads, max_total_threads);
    let policies = [options.io, options.async_compute, options.compute];
    let minimums = policies.map(TaskPoolThreadAssignmentPolicy::minimum_threads);
    let maximums = policies.map(TaskPoolThreadAssignmentPolicy::maximum_threads);
    let minimum_pool_threads = minimums
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .expect("task pool minimum thread counts must fit in usize");
    let maximum_pool_threads = maximums.into_iter().fold(0_usize, usize::saturating_add);
    let total_threads = requested_threads.clamp(minimum_pool_threads, maximum_pool_threads);

    let mut assignments = [0; 3];
    let mut remaining_threads = total_threads;
    for (index, policy) in policies.into_iter().enumerate() {
        let future_minimum = minimums[index + 1..].iter().copied().sum::<usize>();
        let future_maximum = maximums[index + 1..]
            .iter()
            .copied()
            .fold(0_usize, usize::saturating_add);
        let minimum = minimums[index].max(remaining_threads.saturating_sub(future_maximum));
        let maximum = maximums[index].min(
            remaining_threads
                .checked_sub(future_minimum)
                .expect("feasible task pool assignment must retain future minimums"),
        );
        let assigned = policy
            .desired_threads(total_threads)
            .clamp(minimum, maximum);
        assignments[index] = assigned;
        remaining_threads -= assigned;
    }
    debug_assert_eq!(remaining_threads, 0);

    TaskPoolThreadCounts {
        total_threads,
        io_threads: assignments[0],
        async_compute_threads: assignments[1],
        compute_threads: assignments[2],
    }
}

fn available_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, |value| value.get())
}
