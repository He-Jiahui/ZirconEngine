//! Service registry and core runtime.

mod config_store;
mod contexts;
mod descriptors;
pub mod diagnostics;
mod events;
mod frame_clock;
mod handle;
mod lifecycle;
pub mod modules;
mod runtime;
mod state;
pub mod tasks;
mod time;
mod weak;

pub use contexts::{ModuleContext, PluginContext};
pub use descriptors::{
    DependencySpec, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, PluginDescriptor,
    PluginFactory, RegistryName, ServiceFactory, ServiceObject,
};
pub use events::EventBus;
pub use frame_clock::FrameClock;
pub use handle::CoreHandle;
pub use lifecycle::{LifecycleState, ServiceKind, StartupMode};
pub use modules::{
    DiagnosticsCoreModule, FrameCountModule, LogDiagnosticsModule, LogModule, TasksModule,
    TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use runtime::CoreRuntime;
pub use tasks::{
    JobScheduler, TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TaskPoolReport,
    TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts, TaskPools,
};
pub use time::{
    RuntimeTimeAdvance, RuntimeTimeClocks, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
pub use weak::CoreWeak;

#[cfg(test)]
mod tests;
