//! 启动描述符供 App 配置和原生窗口创建使用；运行期窗口、输出与表面状态由带代数的 Host 契约维护。
//! Neutral window contracts shared by runtime modules and host backends.

#[path = "constants.rs"]
mod constants;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "display_topology/mod.rs"]
mod display_topology;
#[path = "lifecycle_policy.rs"]
mod lifecycle_policy;
#[path = "mode.rs"]
mod mode;
#[path = "monitor_selection.rs"]
mod monitor_selection;
#[path = "native_window_id.rs"]
mod native_window_id;
#[path = "position.rs"]
mod position;
#[path = "present_mode.rs"]
mod present_mode;
#[path = "primary_window_handle.rs"]
mod primary_window_handle;
#[path = "resize_constraints.rs"]
mod resize_constraints;
#[path = "resolution.rs"]
mod resolution;
#[path = "surface_lease/mod.rs"]
mod surface_lease;
#[path = "validation.rs"]
mod validation;
#[path = "video_mode_selection.rs"]
mod video_mode_selection;
#[path = "window_command/mod.rs"]
mod window_command;
#[path = "window_id.rs"]
mod window_id;
#[path = "window_registry_id.rs"]
mod window_registry_id;
#[path = "window_state/mod.rs"]
mod window_state;

pub use constants::{DEFAULT_WINDOW_TITLE, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY};
pub use descriptor::WindowDescriptor;
pub use lifecycle_policy::{WindowExitCondition, WindowLifecyclePolicy};
pub use mode::WindowMode;
pub use monitor_selection::WindowMonitorSelection;
pub use position::WindowPosition;
pub use present_mode::WindowPresentMode;
pub use primary_window_handle::PrimaryWindowHandle;
pub use resize_constraints::WindowResizeConstraints;
pub use resolution::WindowResolution;

// Generation-qualified native window identity.
pub use native_window_id::NativeWindowId;
pub use window_id::WindowId;
pub use window_registry_id::WindowRegistryId;

// Immutable display topology contracts.
pub use display_topology::{
    DisplayColorSpace, DisplayFeatureState, DisplayId, DisplayIdentityError, DisplayKind,
    DisplayLogicalInsets, DisplayLogicalRect, DisplayObservation, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplaySnapshot, DisplayTopologyError,
    DisplayTopologyGeneration, DisplayTopologyReplacement, DisplayTopologyReplacementError,
    DisplayTopologySnapshot,
};

// Native surface lifetime and replacement ownership.
pub(crate) use surface_lease::SurfaceLeaseRetirementPlan;
pub use surface_lease::{
    PreparedSurfaceLease, SurfaceLease, SurfaceLeaseError, SurfaceLeaseGeneration,
    SurfaceLeasePublication, SurfaceLeaseRegistry, SurfaceLeaseRequest,
};

// Host command transport and terminal receipts.
pub use video_mode_selection::{WindowVideoMode, WindowVideoModeSelection};
pub use window_command::{
    WindowCommand, WindowCommandAccepted, WindowCommandHeader, WindowCommandId,
    WindowCommandReceipt, WindowCommandTerminal, WindowObservedGeneration,
};

// Create, requested, observed, effective, and reconciliation contracts.
pub use window_state::{
    WindowCreateGeneration, WindowCreateSnapshot, WindowCreateSpec, WindowDisplayTarget,
    WindowEffectiveGeneration, WindowEffectiveMode, WindowEffectivePlacement,
    WindowEffectiveSnapshot, WindowEffectiveState, WindowEffectiveStateError,
    WindowExclusiveFullscreenFallback, WindowExclusiveFullscreenFallbackReason,
    WindowExclusiveFullscreenRequest, WindowExternalStatePolicy, WindowFocusState,
    WindowFullscreenFallback, WindowLogicalExtent, WindowLogicalPosition, WindowObservedMode,
    WindowObservedSnapshot, WindowObservedState, WindowOcclusionState, WindowPhysicalExtent,
    WindowPlacementRequest, WindowReconciliationAction, WindowReconciliationPolicy,
    WindowRequestedGeneration, WindowRequestedMode, WindowRequestedSnapshot, WindowRequestedState,
    WindowStateField, WindowStateReconciliation, WindowStateResizeConstraints, WindowStateSnapshot,
    WindowStateValidationError, WindowVideoModeRequest, WindowVisibilityState,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
