//! Service registry and core runtime.
//! CoreRuntime 的注册、依赖冻结、生命周期和服务访问入口。
//!
//! 描述符提供声明，CoreHandle 持有可变状态；宿主通过 CoreRuntime 驱动外帧与模块阶段。

#[path = "clock_source.rs"]
mod clock_source;
#[path = "config_store.rs"]
mod config_store;
#[path = "contexts/mod.rs"]
mod contexts;
#[path = "descriptors/mod.rs"]
mod descriptors;
#[path = "diagnostics/mod.rs"]
pub mod diagnostics;
#[path = "error.rs"]
pub(super) mod error;
#[path = "events.rs"]
mod events;
#[path = "frame_clock.rs"]
mod frame_clock;
#[path = "handle/mod.rs"]
mod handle;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "module_lifecycle_observer.rs"]
mod module_lifecycle_observer;
#[path = "modules/mod.rs"]
pub mod modules;
#[path = "random/mod.rs"]
pub mod random;
#[path = "runtime.rs"]
mod runtime;
#[path = "shutdown.rs"]
mod shutdown;
#[path = "state/mod.rs"]
mod state;
#[path = "state_machine/mod.rs"]
pub mod state_machine;
#[path = "tasks/mod.rs"]
pub mod tasks;
#[path = "time.rs"]
mod time;
#[path = "weak.rs"]
mod weak;

pub use clock_source::{ClockSource, ManualClockSource, ManualClockSourceError};
pub use contexts::{ModuleCleanupContext, ModuleContext, PluginContext};
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
pub use handle::{CoreHandle, ServiceCallGuard, ServiceHandle};
pub(crate) use handle::{RegisteredServiceIdentity, ServiceAliasSource};
pub use lifecycle::{
    InitLevel, LifecycleState, ModuleLifecycle, NoopModuleLifecycle, ServiceKind, StartupMode,
};
pub use module_lifecycle_observer::{RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver};
pub use modules::{
    DiagnosticsCoreModule, FrameCountModule, LogDiagnosticsModule, LogModule, TasksModule,
    TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use runtime::{CoreRuntime, ModuleShutdownReport};
pub use shutdown::CoreShutdownError;
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
pub(crate) use state::ServiceEntryFactory;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
