//! Neutral window contracts shared by runtime modules and host backends.

mod constants;
mod descriptor;
mod display_topology;
mod lifecycle_policy;
mod mode;
mod monitor_selection;
mod native_window_id;
mod position;
mod present_mode;
mod primary_window_handle;
mod resize_constraints;
mod resolution;
mod surface_lease;
mod validation;
mod video_mode_selection;
mod window_command;
mod window_id;
mod window_registry_id;
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
mod tests;
