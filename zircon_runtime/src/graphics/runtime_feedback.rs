use crate::graphics::runtime_provider::RuntimeProviderFeedback;
use crate::graphics::VisibilityVirtualGeometryFeedback;

use super::VirtualGeometryGpuCompletion;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VirtualGeometryRuntimeFeedback {
    feedback:
        RuntimeProviderFeedback<VirtualGeometryGpuCompletion, VisibilityVirtualGeometryFeedback>,
    node_and_cluster_cull_page_requests: Vec<u32>,
    evictable_page_ids: Vec<u32>,
    generation: u64,
    cpu_prepare_token: Option<crate::core::framework::render::RenderVirtualGeometryCpuPrepareToken>,
    native_readback_outputs:
        Option<crate::core::framework::render::RenderVirtualGeometryReadbackOutputs>,
}

impl VirtualGeometryRuntimeFeedback {
    pub fn with_native_readback_outputs(
        mut self,
        outputs: crate::core::framework::render::RenderVirtualGeometryReadbackOutputs,
    ) -> Self {
        self.native_readback_outputs = Some(outputs);
        self
    }

    pub fn native_readback_outputs(
        &self,
    ) -> Option<&crate::core::framework::render::RenderVirtualGeometryReadbackOutputs> {
        self.native_readback_outputs.as_ref()
    }

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
            cpu_prepare_token: None,
            native_readback_outputs: None,
        }
    }

    pub fn with_evictable_page_ids(mut self, evictable_page_ids: Vec<u32>) -> Self {
        self.evictable_page_ids = evictable_page_ids;
        self
    }

    pub fn with_cpu_prepare_token(
        mut self,
        token: Option<crate::core::framework::render::RenderVirtualGeometryCpuPrepareToken>,
    ) -> Self {
        self.cpu_prepare_token = token;
        self
    }
    pub fn cpu_prepare_token(
        &self,
    ) -> Option<crate::core::framework::render::RenderVirtualGeometryCpuPrepareToken> {
        self.cpu_prepare_token
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
