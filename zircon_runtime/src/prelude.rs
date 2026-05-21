//! Stable convenience imports for runtime-facing Zircon applications and modules.

pub use crate::core::diagnostics::{
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore,
    DiagnosticStoreSnapshot, HotspotReport, ProfileCaptureConfig, ProfileCounterSnapshot,
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, RuntimeDiagnosticsSnapshot,
};
pub use crate::core::framework::time::{Fixed, FixedStepPlan, Real, Time, Virtual};
pub use crate::core::framework::window::{
    PrimaryWindowHandle, WindowDescriptor, WindowExitCondition, WindowLifecyclePolicy, WindowMode,
    WindowMonitorSelection, WindowPosition, WindowPresentMode, WindowResizeConstraints,
    WindowResolution, WindowVideoMode, WindowVideoModeSelection, DEFAULT_WINDOW_TITLE,
    PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
};
pub use crate::core::modules::{
    DiagnosticsCoreModule, FrameCountModule, LogDiagnosticsModule, LogModule, TasksModule,
    TimeModule, DIAGNOSTICS_CORE_MODULE_NAME, FRAME_COUNT_MODULE_NAME, LOG_DIAGNOSTICS_MODULE_NAME,
    LOG_MODULE_NAME, TASKS_MODULE_NAME, TIME_MODULE_NAME,
};
pub use crate::core::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use crate::core::tasks::{
    TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TaskPoolReport,
    TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts, TaskPools,
};
pub use crate::core::{
    CoreError, CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, EngineEvent,
    EventBus, FrameClock, JobScheduler, LifecycleState, ManagerDescriptor, ModuleContext,
    ModuleDescriptor, PluginContext, PluginDescriptor, PluginFactory, RegistryName,
    RuntimeTimeAdvance, RuntimeTimeClocks, ServiceFactory, ServiceKind, StartupMode, ZirconError,
    TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC, TIME_FRAME_COUNT_DIAGNOSTIC,
    TIME_FRAME_TIME_DIAGNOSTIC,
};
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
pub use crate::input::{
    ButtonInputState, DefaultInputManager, FileDragDropEvent, GamepadAxis, GamepadAxisState,
    GamepadButton, GamepadConnectionInfo, GamepadId, ImeCursorArea, ImeCursorRange,
    ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText, InputButton,
    InputConfig, InputDriver, InputEvent, InputEventRecord, InputFrameSnapshot, InputModule,
    InputSnapshot, MouseScrollUnit, MouseWheelEvent, TouchPhase, TouchPoint, WindowStatusEvent,
    WindowTheme, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME, INPUT_MODULE_NAME,
    LEGACY_PIXEL_SCROLL_SCALE,
};
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
pub use crate::plugin::{
    EditorCoreProfile, PluginMaturity, RuntimeCoreProfile, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityReport, RuntimeProfileDescriptor, RuntimeProfileId,
    RuntimeProfilePluginSelection,
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
