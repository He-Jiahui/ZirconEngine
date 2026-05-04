use crate::core::framework::render::RenderPreparedRuntimeSidebands;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_prepared_runtime_sidebands(
        mut self,
        sidebands: RenderPreparedRuntimeSidebands,
    ) -> Self {
        self.prepared_runtime_sidebands = sidebands;
        self
    }
}
