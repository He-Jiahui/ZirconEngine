//! Core runtime, lifecycle management, service registry, and shared runtime primitives.

pub mod runtime;

pub mod framework;
pub mod manager;
pub mod math;
pub mod resource;

pub use framework::error::{CoreError, ZirconError};
pub use framework::events::EngineEvent;
pub use framework::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use runtime::diagnostics;
pub use runtime::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DiagnosticsCoreModule, DriverDescriptor,
    EventBus, FrameClock, FrameCountModule, JobScheduler, LifecycleState, LogDiagnosticsModule,
    LogModule, ManagerDescriptor, ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor,
    PluginFactory, RegistryName, RuntimeTimeAdvance, RuntimeTimeClocks, ServiceFactory,
    ServiceKind, StartupMode, TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions,
    TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts,
    TaskPools, TasksModule, TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME,
    LOG_DIAGNOSTICS_MODULE_NAME, LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_FIXED_STEPS_DIAGNOSTIC,
    TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC, TIME_MODULE_NAME,
};
