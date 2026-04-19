use crate::{PassFlags, QueueLane, RenderPassId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPass {
    pub id: RenderPassId,
    pub name: String,
    pub queue: QueueLane,
    pub flags: PassFlags,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraph {
    name: String,
    passes: Vec<CompiledRenderPass>,
}

impl CompiledRenderGraph {
    pub(crate) fn new(name: String, passes: Vec<CompiledRenderPass>) -> Self {
        Self { name, passes }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }
}
