//! Runtime-owned task helpers and execution primitives.

mod diagnostics;
mod bounded_keyed_io;
mod job_handle;
mod job_scheduler;
mod parallel_for;
mod pool;
mod pools;
mod report;
mod thread_assignment;
mod timer;

use std::thread::{self, JoinHandle};

use crate::core::{CoreError, CoreResult};

pub use crate::core::framework::tasks::{TaskPoolDescriptor, TaskPoolKind};
use diagnostics::JobSchedulerDiagnosticsState;
pub use diagnostics::{
    TASKS_ACTIVE_DIAGNOSTIC, TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC, TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
};
pub use bounded_keyed_io::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoCancelError, BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure,
    BoundedKeyedIoFence, BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard,
    BoundedKeyedIoTerminal,
    BoundedKeyedIoTicket, BoundedKeyedIoWaitResult, BoundedKeyedIoWork,
    BoundedKeyedIoWorkDeadline, GlobalAdmissionEpoch,
};
pub use job_handle::JobHandle;
pub use job_scheduler::JobScheduler;
pub use parallel_for::parallel_for;
pub use pool::TaskPool;
pub use pools::{TaskPoolThreadCounts, TaskPools};
pub use report::{JobSchedulerReport, TaskPoolReport, TaskPoolReportEntry};
pub use thread_assignment::{TaskPoolOptions, TaskPoolThreadAssignmentPolicy};
pub(crate) use timer::{TaskTimer, TaskTimerSubscription};

pub fn spawn_named_thread<F, T>(name: impl Into<String>, task: F) -> CoreResult<JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let name = name.into();
    thread::Builder::new()
        .name(name.clone())
        .spawn(task)
        .map_err(|error| CoreError::ThreadSpawn(format!("{name}: {error}")))
}
