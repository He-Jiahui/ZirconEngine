//! Stable convenience imports for runtime-facing Zircon applications and modules.

pub use crate::core::diagnostics::{
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore,
    DiagnosticStoreSnapshot, RuntimeDiagnosticsSnapshot,
};
pub use crate::core::framework::time::{Fixed, FixedStepPlan, Real, Time, Virtual};
pub use crate::core::modules::{
    DiagnosticsCoreModule, FrameCountModule, TasksModule, TimeModule, DIAGNOSTICS_CORE_MODULE_NAME,
    FRAME_COUNT_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use crate::core::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use crate::core::{
    CoreError, CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, EngineEvent,
    EventBus, FrameClock, JobScheduler, LifecycleState, ManagerDescriptor, ModuleContext,
    ModuleDescriptor, PluginContext, PluginDescriptor, PluginFactory, RegistryName, ServiceFactory,
    ServiceKind, StartupMode, ZirconError,
};
pub use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
    DiagnosticLogLevelParseError, DiagnosticLogModuleFilter, DIAGNOSTIC_LOG_FILTER_ENV,
    DIAGNOSTIC_LOG_LEVEL_ENV,
};
pub use crate::engine_module::{
    dependency_on, driver_contract, factory, manager_contract, module_context, plugin_context,
    plugin_contract, plugin_factory, qualified_name, DriverContract, EngineDriver, EngineManager,
    EngineModule, EnginePlugin, EngineService, ManagerContract, PluginContract,
};
pub use crate::foundation::{FoundationModule, FOUNDATION_MODULE_NAME};
pub use crate::plugin::{
    RuntimePluginAvailabilityEntry, RuntimePluginAvailabilityReport, RuntimeProfileDescriptor,
    RuntimeProfileId, RuntimeProfilePluginSelection,
};
pub use crate::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
    runtime_core_modules, runtime_modules_for_runtime_profile,
    runtime_modules_for_runtime_profile_with_plugin_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports, RuntimeModuleLoadReport,
    RuntimePluginId, RuntimeRequiredPluginMissing, RuntimeTargetMode,
};
