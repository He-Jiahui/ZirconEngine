use super::hybrid_gi_prepare::HybridGiPrepareFrame;
use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_hybrid_gi_prepare(mut self, prepare: Option<HybridGiPrepareFrame>) -> Self {
        self.hybrid_gi_prepare = prepare;
        self
    }
}
