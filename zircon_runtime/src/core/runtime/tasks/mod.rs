//! Runtime-owned task helpers and execution primitives.

mod job_scheduler;
mod pool;
mod pools;
mod report;
mod thread_assignment;

use std::thread::{self, JoinHandle};

use crate::core::ZirconError;

pub use crate::core::framework::tasks::{TaskPoolDescriptor, TaskPoolKind};
pub use job_scheduler::JobScheduler;
pub use pool::TaskPool;
pub use pools::{TaskPoolThreadCounts, TaskPools};
pub use report::{TaskPoolReport, TaskPoolReportEntry};
pub use thread_assignment::{TaskPoolOptions, TaskPoolThreadAssignmentPolicy};

pub fn spawn_named_thread<F, T>(
    name: impl Into<String>,
    task: F,
) -> Result<JoinHandle<T>, ZirconError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let name = name.into();
    thread::Builder::new()
        .name(name.clone())
        .spawn(task)
        .map_err(|error| ZirconError::ThreadSpawn(format!("{name}: {error}")))
}
