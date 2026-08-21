use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::{
    ZrRuntimeViewportHandle, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

#[derive(Clone, Debug)]
pub struct ViewportState {
    pub size: UVec2,
    runtime_viewport: ZrRuntimeViewportHandle,
}

impl ViewportState {
    pub fn new(size: UVec2) -> Self {
        Self::with_runtime_viewport(size, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1)
    }

    pub(crate) fn with_runtime_viewport(
        size: UVec2,
        runtime_viewport: ZrRuntimeViewportHandle,
    ) -> Self {
        Self {
            size: UVec2::new(size.x.max(1), size.y.max(1)),
            runtime_viewport,
        }
    }

    pub(crate) fn runtime_viewport(&self) -> ZrRuntimeViewportHandle {
        self.runtime_viewport
    }

    pub(crate) fn resize(&mut self, size: UVec2) {
        self.size = UVec2::new(size.x.max(1), size.y.max(1));
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new(UVec2::new(960, 540))
    }
}
