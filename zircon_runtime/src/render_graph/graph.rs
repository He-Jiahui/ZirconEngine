use super::types::{PassFlags, QueueLane, RenderGraphResourceLifetime, RenderPassId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPass {
    pub id: RenderPassId,
    pub name: String,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub culled: bool,
    pub executor_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraph {
    name: String,
    passes: Vec<CompiledRenderPass>,
    resource_lifetimes: Vec<RenderGraphResourceLifetime>,
}

impl CompiledRenderGraph {
    pub(crate) fn new(
        name: String,
        passes: Vec<CompiledRenderPass>,
        resource_lifetimes: Vec<RenderGraphResourceLifetime>,
    ) -> Self {
        Self {
            name,
            passes,
            resource_lifetimes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }

    pub fn resource_lifetimes(&self) -> &[RenderGraphResourceLifetime] {
        &self.resource_lifetimes
    }

    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        self.passes
            .iter()
            .filter(|pass| pass.queue == queue && !pass.culled)
            .count()
    }
}
