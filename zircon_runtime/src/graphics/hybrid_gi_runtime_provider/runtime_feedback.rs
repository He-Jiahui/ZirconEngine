use crate::graphics::VisibilityHybridGiFeedback;

use super::HybridGiGpuCompletion;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimeFeedback {
    gpu_completion: Option<HybridGiGpuCompletion>,
    visibility_feedback: Option<VisibilityHybridGiFeedback>,
    evictable_probe_ids: Vec<u32>,
}

impl HybridGiRuntimeFeedback {
    pub fn new(
        gpu_completion: Option<HybridGiGpuCompletion>,
        visibility_feedback: Option<VisibilityHybridGiFeedback>,
    ) -> Self {
        Self {
            gpu_completion,
            visibility_feedback,
            evictable_probe_ids: Vec::new(),
        }
    }

    pub fn with_evictable_probe_ids(mut self, evictable_probe_ids: Vec<u32>) -> Self {
        self.evictable_probe_ids = evictable_probe_ids;
        self
    }

    pub fn gpu_completion(&self) -> Option<&HybridGiGpuCompletion> {
        self.gpu_completion.as_ref()
    }

    pub fn visibility_feedback(&self) -> Option<&VisibilityHybridGiFeedback> {
        self.visibility_feedback.as_ref()
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }
}
