//! Service registry and core runtime.

mod config_store;
mod contexts;
mod descriptors;
pub mod diagnostics;
mod events;
mod frame_clock;
mod handle;
mod lifecycle;
mod module_lifecycle_observer;
pub mod modules;
mod runtime;
mod state;
pub mod tasks;
mod time;
mod weak;

pub use contexts::{ModuleContext, PluginContext};
pub use descriptors::{
    sort_module_activation_order, DependencySpec, DriverDescriptor, ManagerDescriptor,
    ModuleDependencySpec, ModuleDescriptor, PluginDescriptor, PluginFactory, RegistryName,
    ServiceFactory, ServiceObject,
};
pub use events::EventBus;
pub use frame_clock::FrameClock;
pub use handle::CoreHandle;
pub(crate) use handle::RegisteredServiceIdentity;
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
    parallel_for, JobHandle, JobScheduler, JobSchedulerReport, TaskPool, TaskPoolDescriptor,
    TaskPoolKind, TaskPoolOptions, TaskPoolReport, TaskPoolReportEntry,
    TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts, TaskPools, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC,
    TASKS_SCHEDULED_DIAGNOSTIC,
};
pub use time::{
    RuntimeTimeAdvance, RuntimeTimeClocks, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
pub use weak::CoreWeak;

#[cfg(test)]
mod tests;
