use std::collections::VecDeque;

use super::error::RenderGraphError;
use super::graph::{CompiledRenderGraph, CompiledRenderPass};
use super::types::{
    ExternalResource, PassFlags, QueueLane, RenderPassId, TransientBuffer, TransientTexture,
};

#[derive(Clone, Debug)]
struct RenderPassNode {
    id: RenderPassId,
    name: String,
    queue: QueueLane,
    flags: PassFlags,
    dependencies: Vec<RenderPassId>,
}

#[derive(Clone, Debug)]
pub struct RenderGraphBuilder {
    name: String,
    passes: Vec<RenderPassNode>,
    next_transient_texture: usize,
    next_transient_buffer: usize,
    next_external_resource: usize,
}

impl RenderGraphBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: Vec::new(),
            next_transient_texture: 0,
            next_transient_buffer: 0,
            next_external_resource: 0,
        }
    }

    pub fn add_pass(&mut self, name: impl Into<String>, queue: QueueLane) -> RenderPassId {
        let id = RenderPassId(self.passes.len());
        self.passes.push(RenderPassNode {
            id,
            name: name.into(),
            queue,
            flags: PassFlags::default(),
            dependencies: Vec::new(),
        });
        id
    }

    pub fn add_dependency(
        &mut self,
        before: RenderPassId,
        after: RenderPassId,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(before)?;
        self.ensure_pass(after)?;
        let pass = &mut self.passes[after.0];
        if !pass.dependencies.contains(&before) {
            pass.dependencies.push(before);
        }
        Ok(())
    }

    pub fn create_transient_texture(&mut self) -> TransientTexture {
        let id = self.next_transient_texture;
        self.next_transient_texture += 1;
        TransientTexture(id)
    }

    pub fn create_transient_buffer(&mut self) -> TransientBuffer {
        let id = self.next_transient_buffer;
        self.next_transient_buffer += 1;
        TransientBuffer(id)
    }

    pub fn import_external_resource(&mut self) -> ExternalResource {
        let id = self.next_external_resource;
        self.next_external_resource += 1;
        ExternalResource(id)
    }

    pub fn compile(self) -> Result<CompiledRenderGraph, RenderGraphError> {
        let mut indegree = vec![0_usize; self.passes.len()];
        let mut dependents = vec![Vec::new(); self.passes.len()];

        for pass in &self.passes {
            indegree[pass.id.0] = pass.dependencies.len();
            for dependency in &pass.dependencies {
                dependents[dependency.0].push(pass.id);
            }
        }

        let mut ready = VecDeque::new();
        for pass in &self.passes {
            if indegree[pass.id.0] == 0 {
                ready.push_back(pass.id);
            }
        }

        let mut ordered = Vec::with_capacity(self.passes.len());
        while let Some(id) = ready.pop_front() {
            let pass = &self.passes[id.0];
            ordered.push(CompiledRenderPass {
                id,
                name: pass.name.clone(),
                queue: pass.queue,
                flags: pass.flags,
            });

            for dependent in &dependents[id.0] {
                indegree[dependent.0] -= 1;
                if indegree[dependent.0] == 0 {
                    ready.push_back(*dependent);
                }
            }
        }

        if ordered.len() != self.passes.len() {
            return Err(RenderGraphError::CycleDetected {
                graph_name: self.name,
            });
        }

        Ok(CompiledRenderGraph::new(self.name, ordered))
    }

    fn ensure_pass(&self, id: RenderPassId) -> Result<(), RenderGraphError> {
        if id.0 >= self.passes.len() {
            return Err(RenderGraphError::UnknownPass { pass: id.0 });
        }
        Ok(())
    }
}
