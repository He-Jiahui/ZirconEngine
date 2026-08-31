//! Runtime-owned task helpers and execution primitives.

mod bounded_keyed_io;
mod bounded_stream_io;
mod callback_dispatcher;
mod diagnostic_observation;
mod diagnostics;
mod job_handle;
mod job_scheduler;
mod parallel_for;
mod pool;
mod pools;
mod report;
mod retained_byte_budget;
mod task_cancellation_policy;
mod task_descriptor;
mod task_graph;
mod task_id;
mod task_pool_descriptor;
mod task_pool_kind;
mod task_state;
mod task_status;
mod thread_assignment;
mod timer;

use std::thread::{self, JoinHandle};

use crate::core::{CoreError, CoreResult};

pub use bounded_keyed_io::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoCancelError, BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure,
    BoundedKeyedIoFence, BoundedKeyedIoKey, BoundedKeyedIoLane, BoundedKeyedIoLimits,
    BoundedKeyedIoShutdownGuard, BoundedKeyedIoShutdownReport, BoundedKeyedIoTerminal,
    BoundedKeyedIoTicket, BoundedKeyedIoWaitResult, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline,
    GlobalAdmissionEpoch,
};
pub use bounded_stream_io::{
    BoundedStreamIoAdmissionError, BoundedStreamIoBatch, BoundedStreamIoCapture,
    BoundedStreamIoDiagnostics, BoundedStreamIoDrainBudget, BoundedStreamIoFailure,
    BoundedStreamIoLane, BoundedStreamIoLaneDiagnostics, BoundedStreamIoLimitError,
    BoundedStreamIoLimits, BoundedStreamIoReader, BoundedStreamIoRecord, BoundedStreamIoStreamId,
    DEFAULT_BOUNDED_STREAM_IO_MAX_CONCURRENT_READERS, DEFAULT_BOUNDED_STREAM_IO_MAX_LINE_BYTES,
    DEFAULT_BOUNDED_STREAM_IO_QUEUE_BYTE_CAPACITY, DEFAULT_BOUNDED_STREAM_IO_QUEUE_ENTRY_CAPACITY,
    DEFAULT_BOUNDED_STREAM_IO_READ_CHUNK_BYTES,
};
pub use diagnostic_observation::{
    TaskDiagnosticBatch, TaskDiagnosticCursor, TaskDiagnosticIdentity, TaskDiagnosticKind,
    TaskDiagnosticObservation, TaskDiagnosticSeverity, TaskDiagnosticSource,
    MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES, TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES,
    TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};
use diagnostics::JobSchedulerDiagnosticsState;
pub use diagnostics::{
    TASKS_ACTIVE_DIAGNOSTIC, TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAITING_DIAGNOSTIC, TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC,
    TASKS_EXECUTION_MS_DIAGNOSTIC, TASKS_EXECUTION_SAMPLES_DIAGNOSTIC,
    TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC, TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_MS_DIAGNOSTIC, TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC,
    TASKS_SCHEDULED_DIAGNOSTIC,
};
pub use job_handle::JobHandle;
pub use job_scheduler::JobScheduler;
pub use parallel_for::parallel_for;
pub use parallel_for::parallel_map_indices;
pub use parallel_for::parallel_map_ordered;
pub(super) use pool::TaskPoolSubmission;
pub use pool::{TaskPool, TaskPoolBuildError};
pub use pools::{TaskPoolThreadCounts, TaskPools};
pub use report::{JobSchedulerReport, TaskPoolReport, TaskPoolReportEntry};
pub use retained_byte_budget::{
    RetainedByteBudget, RetainedByteBudgetDiagnostics, RetainedByteBudgetError, RetainedByteLease,
};
pub use task_cancellation_policy::TaskCancellationPolicy;
pub use task_descriptor::TaskDescriptor;
pub use task_graph::{
    EngineTaskGraph, EngineTaskGraphInitError, EngineTaskGraphOptions, TaskCancellationToken,
    TaskGraphAdmissionError, TaskGraphScope, TaskGraphScopeCensus, TaskGraphScopeDescriptor,
    TaskGraphShutdownError, TaskGraphShutdownReport, TaskGraphWorkerInventory,
    TaskGraphWorkerShutdownCensus, TaskHandle, DEFAULT_TASK_GRAPH_SCOPE_TASK_CAPACITY,
};
pub use task_id::TaskId;
pub use task_pool_descriptor::TaskPoolDescriptor;
pub use task_pool_kind::TaskPoolKind;
pub use task_state::TaskState;
pub use task_status::TaskStatus;
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
