use std::num::NonZeroU32;
use std::sync::Mutex;

use crate::core::framework::platform::{
    PlatformHostBackend, PlatformHostBackendKind, PlatformHostDescriptor, PlatformHostEvidence,
    PlatformHostObservedCapabilities, PlatformHostQuiesceRequest, PlatformHostThreadAffinity,
};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayLogicalRect, DisplayObservation, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplaySnapshot, DisplayTopologyGeneration,
    DisplayTopologySnapshot, WindowCreateSpec, WindowDisplayTarget, WindowEffectiveMode,
    WindowEffectivePlacement, WindowEffectiveState, WindowFocusState, WindowLogicalExtent,
    WindowLogicalPosition, WindowObservedMode, WindowObservedState, WindowOcclusionState,
    WindowPhysicalExtent, WindowPlacementRequest, WindowRequestedMode, WindowRequestedState,
    WindowStateResizeConstraints, WindowVisibilityState,
};

#[derive(Debug)]
pub(super) struct RecordingPlatformHostBackend {
    requests: Mutex<Vec<PlatformHostQuiesceRequest>>,
}

impl RecordingPlatformHostBackend {
    pub(super) fn new() -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn request_count(&self) -> usize {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

impl PlatformHostBackend for RecordingPlatformHostBackend {
    fn descriptor(&self) -> PlatformHostDescriptor {
        PlatformHostDescriptor::new(
            PlatformHostBackendKind::Winit,
            PlatformHostThreadAffinity::MainThreadOnly,
        )
    }

    fn request_quiesce(
        &self,
        request: PlatformHostQuiesceRequest,
    ) -> Result<(), crate::core::framework::platform::PlatformHostBackendRequestError> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(request);
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct RejectingPlatformHostBackend;

impl PlatformHostBackend for RejectingPlatformHostBackend {
    fn descriptor(&self) -> PlatformHostDescriptor {
        PlatformHostDescriptor::new(
            PlatformHostBackendKind::Winit,
            PlatformHostThreadAffinity::MainThreadOnly,
        )
    }

    fn request_quiesce(
        &self,
        _request: PlatformHostQuiesceRequest,
    ) -> Result<(), crate::core::framework::platform::PlatformHostBackendRequestError> {
        Err(crate::core::framework::platform::PlatformHostBackendRequestError::RequestQueueClosed)
    }
}

pub(super) fn observed_host_evidence() -> PlatformHostEvidence {
    PlatformHostEvidence::new(PlatformHostObservedCapabilities::new(true, true, true))
        .with_backend_version("test-host")
        .expect("test evidence stays inside the version bound")
}

pub(super) fn display_topology(generation: u64, display: &str) -> DisplayTopologySnapshot {
    let display_id = DisplayId::new(DisplayKind::PhysicalOutput, display)
        .expect("display fixture identity is valid");
    let display = DisplaySnapshot::new(
        display_id.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(
                0,
                0,
                NonZeroU32::new(1920).expect("fixture width is nonzero"),
                NonZeroU32::new(1080).expect("fixture height is nonzero"),
            ),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1920.0, 1080.0)
                .expect("fixture usable bounds are valid"),
            scale_factor: 1.0,
            refresh_rate_millihertz: NonZeroU32::new(60_000),
            orientation: DisplayOrientation::Landscape,
            safe_area: None,
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect("fixture display snapshot is valid");
    DisplayTopologySnapshot::new(
        DisplayTopologyGeneration::new(generation).expect("generation is nonzero"),
        vec![display],
        Some(display_id),
    )
    .expect("fixture topology is valid")
}

pub(super) fn command_display() -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, "edid:command-panel")
        .expect("command fixture display identity is valid")
}

pub(super) fn command_topology_generation() -> DisplayTopologyGeneration {
    DisplayTopologyGeneration::new(3).expect("command fixture topology generation is nonzero")
}

pub(super) fn command_physical_extent() -> WindowPhysicalExtent {
    WindowPhysicalExtent::new(1920, 1080).expect("command fixture physical extent is valid")
}

pub(super) fn command_logical_extent() -> WindowLogicalExtent {
    WindowLogicalExtent::new(960.0, 540.0).expect("command fixture logical extent is valid")
}

pub(super) fn command_constraints() -> WindowStateResizeConstraints {
    WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("command fixture minimum is valid"),
        Some(command_logical_extent()),
    )
    .expect("command fixture constraints are valid")
}

pub(super) fn command_requested_state(title: &str) -> WindowRequestedState {
    WindowRequestedState::new(
        title,
        WindowPlacementRequest::CenteredOn(WindowDisplayTarget::Display(command_display())),
        WindowRequestedMode::Windowed,
        command_physical_extent(),
        command_constraints(),
        true,
        true,
        true,
    )
}

pub(super) fn command_window_state(
    title: &str,
) -> (WindowCreateSpec, WindowObservedState, WindowEffectiveState) {
    let requested = command_requested_state(title);
    let observed = WindowObservedState::new(
        command_display(),
        command_topology_generation(),
        command_physical_extent(),
        command_logical_extent(),
        WindowLogicalPosition::new(80.0, 40.0).expect("command fixture position is valid"),
        2.0,
        WindowObservedMode::Windowed,
        WindowFocusState::Focused,
        WindowVisibilityState::Visible,
        WindowOcclusionState::Unoccluded,
    )
    .expect("command fixture observed state is valid");
    let effective = WindowEffectiveState::new(
        title,
        WindowEffectivePlacement::new(
            command_display(),
            WindowLogicalPosition::new(80.0, 40.0).expect("command fixture position is valid"),
        ),
        WindowEffectiveMode::Windowed,
        command_physical_extent(),
        command_constraints(),
        true,
        true,
        true,
        command_topology_generation(),
    )
    .expect("command fixture effective state is valid");
    (
        WindowCreateSpec::new(requested, command_topology_generation()),
        observed,
        effective,
    )
}
