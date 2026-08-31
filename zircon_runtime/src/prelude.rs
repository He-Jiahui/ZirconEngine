//! Stable convenience imports for runtime-facing Zircon applications and modules.

pub use crate::asset::prelude::*;
pub use crate::builtin::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
    runtime_core_modules, runtime_modules_for_compiled_project_plugin_plan,
    runtime_modules_for_runtime_profile,
    runtime_modules_for_runtime_profile_compiled_project_plugin_plan,
    runtime_modules_for_runtime_profile_with_plugin_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports,
};
pub use crate::core::diagnostics::{
    CounterHotspotEntry, CounterHotspotReport, DiagnosticMeasurement, DiagnosticPath,
    DiagnosticSeriesSnapshot, DiagnosticStore, DiagnosticStoreSnapshot, HotspotReport,
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSnapshot,
    ProfileSpanSnapshot, RuntimeDiagnosticsSnapshot,
};
pub use crate::core::framework::project::RuntimeProfileId;
pub use crate::core::framework::time::{
    ClockDomainDescriptor, ClockDomainId, ClockDomainMarker, ClockDomainRegistry, ClockDomainStamp,
    ClockDomainUnit, Fixed, FixedStepPlan, MonotonicReal, ProductTimePolicy,
    ProductTimePolicyError, ProductTimeProfile, Time, TimePolicy, TimePolicyError,
    TimePolicyTransaction, Virtual,
};
pub use crate::core::framework::window::{
    PrimaryWindowHandle, WindowDescriptor, WindowExitCondition, WindowLifecyclePolicy, WindowMode,
    WindowMonitorSelection, WindowPosition, WindowPresentMode, WindowResizeConstraints,
    WindowResolution, WindowVideoMode, WindowVideoModeSelection, DEFAULT_WINDOW_TITLE,
    PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
pub use crate::core::runtime::modules::{
    DiagnosticsCoreModule, FrameCountModule, LogDiagnosticsModule, LogModule, TasksModule,
    TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use crate::core::runtime::state_machine::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use crate::core::runtime::tasks::{
    parallel_for, EngineTaskGraph, EngineTaskGraphInitError, EngineTaskGraphOptions, JobHandle,
    JobSchedulerReport, TaskCancellationPolicy, TaskCancellationToken, TaskDescriptor,
    TaskGraphAdmissionError, TaskGraphScope, TaskGraphScopeCensus, TaskGraphScopeDescriptor,
    TaskGraphShutdownError, TaskGraphShutdownReport, TaskGraphWorkerInventory, TaskHandle, TaskId,
    TaskPool, TaskPoolBuildError, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions,
    TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts,
    TaskPools, TaskState, TaskStatus, TASKS_ACTIVE_DIAGNOSTIC, TASKS_CANCELLED_DIAGNOSTIC,
    TASKS_COMPLETED_DIAGNOSTIC, TASKS_DEPENDENCY_WAITING_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC,
    TASKS_PANICKED_DIAGNOSTIC, TASKS_QUEUED_DIAGNOSTIC, TASKS_QUEUE_WAIT_MS_DIAGNOSTIC,
    TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC, TASKS_SCHEDULED_DIAGNOSTIC,
};
pub use crate::core::{
    ClockDiscontinuity, ClockLifecycleTransition, ClockSource, CoreError, CoreHandle, CoreResult,
    CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, EngineEvent, EventBus, FrameClock,
    FrameClockFirstTickPolicy, FrameClockRebaseCause, FrameClockRebaseReceipt,
    FrameTimeDiscontinuity, FrameTimeSnapshot, JobScheduler, LifecycleState, ManagerDescriptor,
    ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor, PluginFactory,
    ProductTimePolicies, ProductTimePolicyDigest, RegistryName, ServiceFactory, ServiceKind,
    StartupMode, TimePolicyReceipt, TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC,
    TIME_FRAME_TIME_DIAGNOSTIC,
};
#[cfg(feature = "diagnostic-log")]
pub use crate::diagnostic_log::{
    format_diagnostic_store_snapshot, write_diagnostic_store_snapshot, DiagnosticLogFilter,
    DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogLevelParseError,
    DiagnosticLogLocation, DiagnosticLogModuleFilter, DiagnosticLogSettings,
    DiagnosticStoreLogSchedule, LogSettings, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT, DIAGNOSTIC_LOG_ENV,
    DIAGNOSTIC_LOG_FILTER_ENV, DIAGNOSTIC_LOG_LEVEL_ENV, RUST_LOG_ENV,
};
pub use crate::engine_module::{
    dependency_on, driver_contract, factory, manager_contract, module_context, plugin_context,
    plugin_contract, plugin_factory, qualified_name, DriverContract, EngineDriver, EngineManager,
    EngineModule, EnginePlugin, EngineService, ManagerContract, PluginContract,
};
pub use crate::foundation::{FoundationModule, FOUNDATION_MODULE_NAME};
#[cfg(feature = "graphics")]
pub use crate::graphics::prelude::*;
pub use crate::input::prelude::*;
pub use crate::platform::{
    CapabilityStatus, CursorBoundaryBackend, CursorOptionsBackend, EventLoopPolicy,
    FileDragDropBackend, GamepadBackend, GamepadEventBackend, GamepadRumbleBackend,
    GestureEventBackend, ImeBackend, InputBackend, KeyboardEventBackend, LinuxWindowProtocol,
    MonitorBackend, MouseButtonBackend, MouseWheelBackend, PlatformCapabilityMatrix,
    PlatformCapabilityReport, PlatformConfig, PlatformDriver, PlatformFeatureSelection,
    PlatformManager, PlatformModule, PlatformTarget, PointerPositionBackend, RawMouseMotionBackend,
    TouchEventBackend, WindowBackend, WindowEventBackend, WindowLifecycleBackend,
    WindowMetricsBackend, PLATFORM_CONFIG_KEY, PLATFORM_DRIVER_NAME, PLATFORM_MANAGER_NAME,
    PLATFORM_MODULE_NAME,
};
pub use crate::scene::prelude::*;
#[cfg(feature = "ui")]
pub use crate::ui::prelude::*;
