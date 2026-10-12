//! Shared default plan for one runtime's task pools, Net service, and Jolt world.
//! Dedicated I/O owners are included even when they are normally parked.

use std::sync::OnceLock;

use super::TaskPoolOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineThreadBudget {
    hardware_threads: usize,
    task_worker_threads: usize,
    physics_worker_threads: usize,
}

impl EngineThreadBudget {
    /// Three independent task domains require at least three physical workers.
    /// On hosts below seven logical CPUs the reported plan therefore exceeds
    /// hardware parallelism; it never silently reports a smaller physical sum.
    pub fn for_parallelism(hardware_threads: usize) -> Self {
        let hardware_threads = hardware_threads.max(1);
        let physics_worker_threads = hardware_threads.saturating_sub(7).min(4);
        let reserved = 4 + physics_worker_threads; // frame, Net async, blocking, socket owner
        Self {
            hardware_threads,
            task_worker_threads: hardware_threads.saturating_sub(reserved).max(3),
            physics_worker_threads,
        }
    }

    pub fn process_default() -> Self {
        static BUDGET: OnceLock<EngineThreadBudget> = OnceLock::new();
        *BUDGET.get_or_init(|| {
            Self::for_parallelism(
                std::thread::available_parallelism().map_or(1, |count| count.get()),
            )
        })
    }

    pub const fn hardware_threads(self) -> usize {
        self.hardware_threads
    }
    pub const fn task_worker_threads(self) -> usize {
        self.task_worker_threads
    }
    pub const fn net_async_worker_threads(self) -> usize {
        1
    }
    pub const fn net_blocking_threads(self) -> usize {
        1
    }
    pub const fn net_transport_threads(self) -> usize {
        1
    }
    /// Zero runs Jolt jobs on the calling update thread, without a Jolt worker.
    pub const fn physics_worker_threads(self) -> usize {
        self.physics_worker_threads
    }

    /// Upper bound including the caller's frame thread and Tokio blocking cap.
    /// Audio callback threads, lifecycle timers, and extra runtime instances
    /// have separate owners and must be added by the application host.
    pub const fn planned_threads(self) -> usize {
        self.task_worker_threads + self.physics_worker_threads + 4
    }

    pub fn task_pool_options(self) -> TaskPoolOptions {
        TaskPoolOptions::with_num_threads(self.task_worker_threads)
    }
}

#[cfg(test)]
#[path = "tests/thread_budget.rs"]
mod tests;
