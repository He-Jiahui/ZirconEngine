use zircon_runtime_interface::ZrRuntimeViewportHandle;

use crate::core::framework::window::{DisplayId, DisplayTopologyGeneration, WindowId};

/// The platform facts a surface preparation must bind together. Graphics
/// negotiation adds format and present policy later; this platform contract
/// keeps native-window, viewport, output, and topology authority aligned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceLeaseRequest {
    window: WindowId,
    viewport: ZrRuntimeViewportHandle,
    output: DisplayId,
    topology_generation: DisplayTopologyGeneration,
}

impl SurfaceLeaseRequest {
    pub(crate) const fn new(
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
        output: DisplayId,
        topology_generation: DisplayTopologyGeneration,
    ) -> Self {
        Self {
            window,
            viewport,
            output,
            topology_generation,
        }
    }

    pub const fn window(&self) -> WindowId {
        self.window
    }

    pub const fn viewport(&self) -> ZrRuntimeViewportHandle {
        self.viewport
    }

    pub fn output(&self) -> &DisplayId {
        &self.output
    }

    pub const fn topology_generation(&self) -> DisplayTopologyGeneration {
        self.topology_generation
    }
}
