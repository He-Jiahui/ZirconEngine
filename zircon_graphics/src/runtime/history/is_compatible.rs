use zircon_math::UVec2;
use zircon_render_server::RenderPipelineHandle;

use crate::FrameHistoryBinding;

use super::viewport_frame_history::ViewportFrameHistory;

impl ViewportFrameHistory {
    pub(crate) fn is_compatible(
        &self,
        viewport_size: UVec2,
        pipeline: RenderPipelineHandle,
        bindings: &[FrameHistoryBinding],
    ) -> bool {
        self.viewport_size == viewport_size
            && self.pipeline == pipeline
            && self.bindings == bindings
    }
}
