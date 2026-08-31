use crate::core::framework::window::{DisplayId, DisplayTopologyGeneration};

use super::{
    WindowLogicalExtent, WindowLogicalPosition, WindowPhysicalExtent, WindowStateField,
    WindowStateValidationError, WindowVideoModeRequest,
};

/// Focus is an OS-owned fact. It is intentionally not represented as a
/// requested or effective platform setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowFocusState {
    Focused,
    Unfocused,
}

/// Visibility facts that are separate from application activation and window
/// focus. The host must publish the actual state it can observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowVisibilityState {
    Visible,
    Hidden,
    Minimized,
}

/// Occlusion is a best-effort platform fact. `Unknown` is explicit when the
/// selected backend cannot observe it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowOcclusionState {
    Unknown,
    Unoccluded,
    Occluded,
}

/// The fullscreen shape reported by the OS. The containing observation owns
/// the current stable display identity; an unavailable video mode is never
/// invented from the requested mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowObservedMode {
    Windowed,
    BorderlessFullscreen,
    ExclusiveFullscreen {
        video_mode: Option<WindowVideoModeRequest>,
    },
}

impl WindowObservedMode {
    pub const fn is_fullscreen(self) -> bool {
        !matches!(self, Self::Windowed)
    }
}

/// Fully observed native-window facts for one platform event or command
/// completion. Its publication generation is owned by the command/event
/// broker, while the snapshot carries the display topology generation used to
/// resolve its current output.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowObservedState {
    display: DisplayId,
    display_topology_generation: DisplayTopologyGeneration,
    physical_extent: WindowPhysicalExtent,
    logical_extent: WindowLogicalExtent,
    logical_position: WindowLogicalPosition,
    scale_factor: f64,
    mode: WindowObservedMode,
    focus: WindowFocusState,
    visibility: WindowVisibilityState,
    occlusion: WindowOcclusionState,
}

impl WindowObservedState {
    pub fn new(
        display: DisplayId,
        display_topology_generation: DisplayTopologyGeneration,
        physical_extent: WindowPhysicalExtent,
        logical_extent: WindowLogicalExtent,
        logical_position: WindowLogicalPosition,
        scale_factor: f64,
        mode: WindowObservedMode,
        focus: WindowFocusState,
        visibility: WindowVisibilityState,
        occlusion: WindowOcclusionState,
    ) -> Result<Self, WindowStateValidationError> {
        validate_scale_factor(scale_factor)?;
        Ok(Self {
            display,
            display_topology_generation,
            physical_extent,
            logical_extent,
            logical_position,
            scale_factor,
            mode,
            focus,
            visibility,
            occlusion,
        })
    }

    pub const fn display(&self) -> &DisplayId {
        &self.display
    }

    pub const fn display_topology_generation(&self) -> DisplayTopologyGeneration {
        self.display_topology_generation
    }

    pub const fn physical_extent(&self) -> WindowPhysicalExtent {
        self.physical_extent
    }

    pub const fn logical_extent(&self) -> WindowLogicalExtent {
        self.logical_extent
    }

    pub const fn logical_position(&self) -> WindowLogicalPosition {
        self.logical_position
    }

    pub const fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub const fn mode(&self) -> WindowObservedMode {
        self.mode
    }

    pub const fn focus(&self) -> WindowFocusState {
        self.focus
    }

    pub const fn visibility(&self) -> WindowVisibilityState {
        self.visibility
    }

    pub const fn occlusion(&self) -> WindowOcclusionState {
        self.occlusion
    }
}

fn validate_scale_factor(scale_factor: f64) -> Result<(), WindowStateValidationError> {
    if !scale_factor.is_finite() {
        return Err(WindowStateValidationError::NonFinite {
            field: WindowStateField::ScaleFactor,
            value: scale_factor,
        });
    }
    if scale_factor <= 0.0 {
        return Err(WindowStateValidationError::NonPositive {
            field: WindowStateField::ScaleFactor,
            value: scale_factor,
        });
    }
    Ok(())
}
