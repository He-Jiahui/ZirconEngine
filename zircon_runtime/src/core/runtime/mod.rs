//! Service registry and core runtime.

mod clock_source;
mod config_store;
mod contexts;
mod descriptors;
pub mod diagnostics;
pub(super) mod error;
mod events;
mod frame_clock;
mod handle;
mod lifecycle;
mod module_lifecycle_observer;
pub mod modules;
pub mod random;
mod runtime;
mod state;
pub mod state_machine;
pub mod tasks;
mod time;
mod weak;

pub use clock_source::{ClockSource, ManualClockSource, ManualClockSourceError};
pub use contexts::{ModuleContext, PluginContext};
pub(crate) use descriptors::FrozenModuleGraph;
pub use descriptors::{
    sort_module_activation_order, DependencySpec, DriverDescriptor, ManagerDescriptor,
    ModuleDependencySpec, ModuleDescriptor, PluginDescriptor, PluginFactory, RegistryName,
    ServiceFactory, ServiceObject,
};
pub use events::EventBus;
pub use frame_clock::{
    ClockDiscontinuity, ClockLifecycleTransition, FrameClock, FrameClockFirstTickPolicy,
    FrameClockRebaseCause, FrameClockRebaseReceipt,
};
pub(crate) use handle::RegisteredServiceIdentity;
pub use handle::{CoreHandle, ServiceCallGuard, ServiceHandle};
pub use lifecycle::{
    InitLevel, LifecycleState, ModuleLifecycle, NoopModuleLifecycle, ServiceKind, StartupMode,
};
pub use module_lifecycle_observer::{RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver};
pub use modules::{
    DiagnosticsCoreModule, FrameCountModule, LogDiagnosticsModule, LogModule, TasksModule,
    TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use runtime::CoreRuntime;
pub use tasks::{
    parallel_for, parallel_map_indices, BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError,
    BoundedKeyedIoCancelAuthority, BoundedKeyedIoCancelError, BoundedKeyedIoDiagnostics,
    BoundedKeyedIoFailure, BoundedKeyedIoFence, BoundedKeyedIoKey, BoundedKeyedIoLane,
    BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard, BoundedKeyedIoShutdownReport,
    BoundedKeyedIoTerminal, BoundedKeyedIoTicket, BoundedKeyedIoWaitResult, BoundedKeyedIoWork,
    BoundedKeyedIoWorkDeadline, EngineTaskGraph, EngineTaskGraphInitError, EngineTaskGraphOptions,
    GlobalAdmissionEpoch, JobHandle, JobScheduler, JobSchedulerReport, RetainedByteBudget,
    RetainedByteBudgetDiagnostics, RetainedByteBudgetError, RetainedByteLease,
    TaskCancellationPolicy, TaskCancellationToken, TaskDescriptor, TaskDiagnosticBatch,
    TaskDiagnosticCursor, TaskDiagnosticIdentity, TaskDiagnosticKind, TaskDiagnosticObservation,
    TaskDiagnosticSeverity, TaskDiagnosticSource, TaskGraphAdmissionError, TaskGraphScope,
    TaskGraphScopeCensus, TaskGraphScopeDescriptor, TaskGraphShutdownError,
    TaskGraphShutdownReport, TaskGraphWorkerInventory, TaskHandle, TaskId, TaskPool,
    TaskPoolBuildError, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TaskPoolReport,
    TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts, TaskPools,
    TaskState, TaskStatus, MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES, TASKS_ACTIVE_DIAGNOSTIC,
    TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES, TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};
pub use time::{
    FrameTimeDiscontinuity, FrameTimeSnapshot, ProductTimePolicies, ProductTimePolicyDigest,
    TimePolicyReceipt, TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC,
    TIME_FRAME_TIME_DIAGNOSTIC,
};
pub use weak::CoreWeak;

#[cfg(test)]
mod tests;
