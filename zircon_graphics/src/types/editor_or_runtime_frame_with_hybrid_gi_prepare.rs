use super::editor_or_runtime_frame::EditorOrRuntimeFrame;
use super::hybrid_gi_prepare::HybridGiPrepareFrame;

impl EditorOrRuntimeFrame {
    pub(crate) fn with_hybrid_gi_prepare(mut self, prepare: Option<HybridGiPrepareFrame>) -> Self {
        self.hybrid_gi_prepare = prepare;
        self
    }
}
