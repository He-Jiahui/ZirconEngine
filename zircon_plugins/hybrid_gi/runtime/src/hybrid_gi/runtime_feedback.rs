use super::HybridGiGpuCompletion;
use zircon_runtime::graphics::VisibilityHybridGiFeedback;

pub(crate) struct HybridGiRuntimeFeedback {
    gpu_completion: Option<HybridGiGpuCompletion>,
    visibility_feedback: Option<VisibilityHybridGiFeedback>,
    evictable_probe_ids: Vec<u32>,
}

impl HybridGiRuntimeFeedback {
    pub(crate) fn new(
        gpu_completion: Option<HybridGiGpuCompletion>,
        visibility_feedback: Option<VisibilityHybridGiFeedback>,
    ) -> Self {
        Self {
            gpu_completion,
            visibility_feedback,
            evictable_probe_ids: Vec::new(),
        }
    }

    pub(crate) fn with_evictable_probe_ids(mut self, evictable_probe_ids: Vec<u32>) -> Self {
        self.evictable_probe_ids = evictable_probe_ids;
        self
    }

    pub(crate) fn gpu_completion(&self) -> Option<&HybridGiGpuCompletion> {
        self.gpu_completion.as_ref()
    }

    pub(crate) fn visibility_feedback(&self) -> Option<&VisibilityHybridGiFeedback> {
        self.visibility_feedback.as_ref()
    }

    pub(crate) fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }
}
