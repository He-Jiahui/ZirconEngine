use super::VirtualGeometryGpuCompletion;
use crate::VisibilityVirtualGeometryFeedback;

pub(in crate::graphics::runtime) struct VirtualGeometryRuntimeFeedback {
    gpu_completion: Option<VirtualGeometryGpuCompletion>,
    node_and_cluster_cull_page_requests: Vec<u32>,
    evictable_page_ids: Vec<u32>,
    visibility_feedback: Option<VisibilityVirtualGeometryFeedback>,
}

impl VirtualGeometryRuntimeFeedback {
    pub(in crate::graphics::runtime) fn new(
        gpu_completion: Option<VirtualGeometryGpuCompletion>,
        node_and_cluster_cull_page_requests: Vec<u32>,
        visibility_feedback: Option<VisibilityVirtualGeometryFeedback>,
    ) -> Self {
        Self {
            gpu_completion,
            node_and_cluster_cull_page_requests,
            evictable_page_ids: Vec::new(),
            visibility_feedback,
        }
    }

    pub(in crate::graphics::runtime) fn with_evictable_page_ids(
        mut self,
        evictable_page_ids: Vec<u32>,
    ) -> Self {
        self.evictable_page_ids = evictable_page_ids;
        self
    }

    pub(in crate::graphics::runtime) fn gpu_completion(
        &self,
    ) -> Option<&VirtualGeometryGpuCompletion> {
        self.gpu_completion.as_ref()
    }

    pub(in crate::graphics::runtime) fn node_and_cluster_cull_page_requests(&self) -> &[u32] {
        &self.node_and_cluster_cull_page_requests
    }

    pub(in crate::graphics::runtime) fn evictable_page_ids(&self) -> &[u32] {
        &self.evictable_page_ids
    }

    pub(in crate::graphics::runtime) fn visibility_feedback(
        &self,
    ) -> Option<&VisibilityVirtualGeometryFeedback> {
        self.visibility_feedback.as_ref()
    }
}
