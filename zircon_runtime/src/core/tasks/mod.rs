//! Runtime-owned task pools for compute, async-compute, and IO work.

mod pool;
mod pools;
mod report;
mod thread_assignment;

pub use crate::core::framework::tasks::{TaskPoolDescriptor, TaskPoolKind};
pub use pool::TaskPool;
pub use pools::{TaskPoolThreadCounts, TaskPools};
pub use report::{TaskPoolReport, TaskPoolReportEntry};
pub use thread_assignment::{TaskPoolOptions, TaskPoolThreadAssignmentPolicy};
