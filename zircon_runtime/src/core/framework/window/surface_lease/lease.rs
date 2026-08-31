use super::{SurfaceLeaseGeneration, SurfaceLeaseRequest};
use crate::core::framework::window::{DisplayId, DisplayTopologyGeneration, WindowId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// An active or prepared authority to bind one viewport to one native-window
/// generation and observed output topology generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceLease {
    request: SurfaceLeaseRequest,
    generation: SurfaceLeaseGeneration,
}

impl SurfaceLease {
    pub(super) const fn new(
        request: SurfaceLeaseRequest,
        generation: SurfaceLeaseGeneration,
    ) -> Self {
        Self {
            request,
            generation,
        }
    }

    pub const fn window(&self) -> WindowId {
        self.request.window()
    }

    pub const fn viewport(&self) -> ZrRuntimeViewportHandle {
        self.request.viewport()
    }

    pub fn output(&self) -> &DisplayId {
        self.request.output()
    }

    pub const fn topology_generation(&self) -> DisplayTopologyGeneration {
        self.request.topology_generation()
    }

    pub const fn generation(&self) -> SurfaceLeaseGeneration {
        self.generation
    }

    pub const fn request(&self) -> &SurfaceLeaseRequest {
        &self.request
    }
}

/// A prepared but not-yet-routable surface lease. The caller creates and
/// configures the graphics surface, drains prior submissions, then publishes
/// this exact candidate atomically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSurfaceLease {
    candidate: SurfaceLease,
}

impl PreparedSurfaceLease {
    pub(super) const fn new(candidate: SurfaceLease) -> Self {
        Self { candidate }
    }

    pub const fn candidate(&self) -> &SurfaceLease {
        &self.candidate
    }

    pub const fn request(&self) -> &SurfaceLeaseRequest {
        self.candidate.request()
    }
}

/// The atomic result of publishing a prepared candidate. `retired` remains
/// valid only for the caller's graphics cleanup and cannot route new work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceLeasePublication {
    current: SurfaceLease,
    retired: Option<SurfaceLease>,
}

impl SurfaceLeasePublication {
    pub(super) const fn new(current: SurfaceLease, retired: Option<SurfaceLease>) -> Self {
        Self { current, retired }
    }

    pub const fn current(&self) -> &SurfaceLease {
        &self.current
    }

    pub const fn retired(&self) -> Option<&SurfaceLease> {
        self.retired.as_ref()
    }
}
