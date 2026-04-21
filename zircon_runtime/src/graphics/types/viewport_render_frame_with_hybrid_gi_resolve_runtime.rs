use super::hybrid_gi_resolve_runtime::HybridGiResolveRuntime;
use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_hybrid_gi_resolve_runtime(
        mut self,
        runtime: Option<HybridGiResolveRuntime>,
    ) -> Self {
        self.hybrid_gi_resolve_runtime = runtime;
        self
    }
}
