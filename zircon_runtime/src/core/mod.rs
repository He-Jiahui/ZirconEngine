//! Core runtime, lifecycle management, service registry, and shared runtime primitives.

pub mod runtime;

pub mod framework;
pub mod manager;
pub mod math;
pub mod resource;

pub use framework::events::EngineEvent;
pub use framework::time::{
    ClockDomainDescriptor, ClockDomainId, ClockDomainMarker, ClockDomainRegistry, ClockDomainStamp,
    ClockDomainUnit, ProductTimePolicy, ProductTimePolicyError, ProductTimeProfile, TimePolicy,
    TimePolicyError, TimePolicyTransaction,
};
pub use runtime::diagnostics;
pub use runtime::error::{CoreError, CoreResult};
pub use runtime::state_machine::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use runtime::{
    parallel_for, parallel_map_indices, sort_module_activation_order, ClockDiscontinuity,
    ClockLifecycleTransition, ClockSource, CoreHandle, CoreRuntime, CoreWeak, DependencySpec,
    DiagnosticsCoreModule, DriverDescriptor, EngineTaskGraph, EngineTaskGraphInitError,
    EngineTaskGraphOptions, EventBus, FrameClock, FrameClockFirstTickPolicy, FrameClockRebaseCause,
    FrameClockRebaseReceipt, FrameCountModule, FrameTimeDiscontinuity, FrameTimeSnapshot,
    InitLevel, JobHandle, JobScheduler, JobSchedulerReport, LifecycleState, LogDiagnosticsModule,
    LogModule, ManagerDescriptor, ManualClockSource, ManualClockSourceError, ModuleContext,
    ModuleDependencySpec, ModuleDescriptor, ModuleLifecycle, NoopModuleLifecycle, PluginContext,
    PluginDescriptor, PluginFactory, ProductTimePolicies, ProductTimePolicyDigest, RegistryName,
    RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver, ServiceFactory, ServiceKind,
    StartupMode, TaskCancellationPolicy, TaskCancellationToken, TaskDescriptor,
    TaskDiagnosticBatch, TaskDiagnosticCursor, TaskDiagnosticIdentity, TaskDiagnosticKind,
    TaskDiagnosticObservation, TaskDiagnosticSeverity, TaskDiagnosticSource,
    TaskGraphAdmissionError, TaskGraphScope, TaskGraphScopeCensus, TaskGraphScopeDescriptor,
    TaskGraphShutdownError, TaskGraphShutdownReport, TaskGraphWorkerInventory, TaskHandle, TaskId,
    TaskPool, TaskPoolBuildError, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions,
    TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts,
    TaskPools, TaskState, TaskStatus, TasksModule, TimeModule, TimePolicyReceipt,
    DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES, TASKS_ACTIVE_DIAGNOSTIC,
    TASKS_CANCELLED_DIAGNOSTIC, TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC, TASKS_MODULE_NAME,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES, TASK_DIAGNOSTIC_RETENTION_CAPACITY, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC, TIME_MODULE_NAME,
};
