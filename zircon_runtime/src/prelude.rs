//! Stable convenience imports for runtime-facing Zircon applications and modules.

pub use crate::asset::prelude::*;
pub use crate::builtin::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
    runtime_core_modules, runtime_modules_for_runtime_profile,
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
pub use crate::core::framework::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
pub use crate::core::framework::time::{Fixed, FixedStepPlan, Real, Time, Virtual};
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
pub use crate::core::runtime::tasks::{
    parallel_for, JobHandle, JobSchedulerReport, TaskPool, TaskPoolDescriptor, TaskPoolKind,
    TaskPoolOptions, TaskPoolReport, TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy,
    TaskPoolThreadCounts, TaskPools, TASKS_COMPLETED_DIAGNOSTIC,
    TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC, TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC,
    TASKS_SCHEDULED_DIAGNOSTIC,
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
pub use crate::graphics::prelude::*;
pub use crate::input::{
    ButtonInputState, CursorGrabMode, CursorHostRequest, CursorPosition, DefaultInputActionManager,
    DefaultInputManager, FileDragDropEvent, GamepadAxis, GamepadAxisInput, GamepadAxisSettings,
    GamepadAxisState, GamepadAxisTransition, GamepadButton, GamepadButtonAxisSettings,
    GamepadButtonSettings, GamepadButtonValueState, GamepadConnectionInfo, GamepadId,
    GamepadRumbleIntensity, GamepadRumbleRequest, ImeCursorArea, ImeCursorRange,
    ImeDeleteSurrounding, ImeEvent, ImeHostRequest, ImePreedit, ImeSurroundingText, InputAction,
    InputActionContext, InputActionEvaluator, InputActionManager, InputActionMap, InputActionState,
    InputAxisBinding, InputAxisDirection, InputBinding, InputButton, InputConfig, InputDriver,
    InputEvent, InputEventRecord, InputFrameSnapshot, InputModule, InputRecording,
    InputRecordingFrame, InputReplayCursor, InputReplayFrameReport, InputSnapshot, MouseScrollUnit,
    MouseWheelEvent, TouchPhase, TouchPoint, WindowStatusEvent, WindowTheme,
    GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEADZONE_LOWER, GAMEPAD_AXIS_DEADZONE_UPPER,
    GAMEPAD_AXIS_LIVEZONE_LOWER, GAMEPAD_AXIS_LIVEZONE_UPPER, GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD,
    GAMEPAD_BUTTON_AXIS_HIGH, GAMEPAD_BUTTON_AXIS_LOW, GAMEPAD_BUTTON_PRESS_THRESHOLD,
    GAMEPAD_BUTTON_RELEASE_THRESHOLD, INPUT_ACTION_MANAGER_NAME, INPUT_DRIVER_NAME,
    INPUT_MANAGER_NAME, INPUT_MODULE_NAME, PIXEL_SCROLL_LINE_DELTA_SCALE,
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
pub use crate::scene::prelude::*;
pub use crate::ui::prelude::*;
