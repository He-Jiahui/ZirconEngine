use crate::graphics::runtime_provider::RuntimeProviderFeedback;
use crate::graphics::VisibilityHybridGiFeedback;

use super::HybridGiGpuCompletion;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimeFeedback {
    feedback: RuntimeProviderFeedback<HybridGiGpuCompletion, VisibilityHybridGiFeedback>,
    evictable_probe_ids: Vec<u32>,
}

impl HybridGiRuntimeFeedback {
    pub fn new(
        gpu_completion: Option<HybridGiGpuCompletion>,
        visibility_feedback: Option<VisibilityHybridGiFeedback>,
    ) -> Self {
        Self {
            feedback: RuntimeProviderFeedback::new(gpu_completion, visibility_feedback),
            evictable_probe_ids: Vec::new(),
        }
    }

    pub fn with_evictable_probe_ids(mut self, evictable_probe_ids: Vec<u32>) -> Self {
        self.evictable_probe_ids = evictable_probe_ids;
        self
    }

    pub fn gpu_completion(&self) -> Option<&HybridGiGpuCompletion> {
        self.feedback.gpu_completion()
    }

    pub fn visibility_feedback(&self) -> Option<&VisibilityHybridGiFeedback> {
        self.feedback.visibility_feedback()
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }
}
