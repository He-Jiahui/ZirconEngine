use super::editor_or_runtime_frame::EditorOrRuntimeFrame;
use super::hybrid_gi_resolve_runtime::HybridGiResolveRuntime;

impl EditorOrRuntimeFrame {
    pub(crate) fn with_hybrid_gi_resolve_runtime(
        mut self,
        runtime: Option<HybridGiResolveRuntime>,
    ) -> Self {
        self.hybrid_gi_resolve_runtime = runtime;
        self
    }
}
