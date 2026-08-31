use crate::graphics::VisibilityVirtualGeometryFeedback;
use crate::graphics::runtime_provider::RuntimeProviderFeedback;

use super::VirtualGeometryGpuCompletion;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryRuntimeFeedback {
    feedback:
        RuntimeProviderFeedback<VirtualGeometryGpuCompletion, VisibilityVirtualGeometryFeedback>,
    node_and_cluster_cull_page_requests: Vec<u32>,
    evictable_page_ids: Vec<u32>,
    generation: u64,
}

impl VirtualGeometryRuntimeFeedback {
    pub fn new(
        gpu_completion: Option<VirtualGeometryGpuCompletion>,
        node_and_cluster_cull_page_requests: Vec<u32>,
        visibility_feedback: Option<VisibilityVirtualGeometryFeedback>,
        generation: u64,
    ) -> Self {
        Self {
            feedback: RuntimeProviderFeedback::new(gpu_completion, visibility_feedback),
            node_and_cluster_cull_page_requests,
            evictable_page_ids: Vec::new(),
            generation,
        }
    }

    pub fn with_evictable_page_ids(mut self, evictable_page_ids: Vec<u32>) -> Self {
        self.evictable_page_ids = evictable_page_ids;
        self
    }

    pub fn gpu_completion(&self) -> Option<&VirtualGeometryGpuCompletion> {
        self.feedback.gpu_completion()
    }

    pub fn node_and_cluster_cull_page_requests(&self) -> &[u32] {
        &self.node_and_cluster_cull_page_requests
    }

    pub fn evictable_page_ids(&self) -> &[u32] {
        &self.evictable_page_ids
    }

    pub fn visibility_feedback(&self) -> Option<&VisibilityVirtualGeometryFeedback> {
        self.feedback.visibility_feedback()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}
