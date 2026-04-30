use crate::render_graph::{PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderPassId};

use super::RenderPassExecutorId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPassExecutionContext {
    pub pass_name: String,
    pub executor_id: RenderPassExecutorId,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
}

impl RenderPassExecutionContext {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(pass_name: impl Into<String>, executor_id: RenderPassExecutorId) -> Self {
        Self::with_graph_metadata(
            pass_name,
            executor_id,
            QueueLane::Graphics,
            PassFlags::default(),
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_graph_metadata(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        flags: PassFlags,
    ) -> Self {
        Self::with_graph_metadata_and_resources(pass_name, executor_id, queue, flags, Vec::new())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_declared_graph_metadata(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
    ) -> Self {
        Self::with_declared_graph_metadata_and_resources(
            pass_name,
            executor_id,
            queue,
            declared_queue,
            flags,
            Vec::new(),
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_graph_metadata_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        flags: PassFlags,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self::with_declared_graph_metadata_and_resources(
            pass_name,
            executor_id,
            queue,
            queue,
            flags,
            resources,
        )
    }

    pub fn with_declared_graph_metadata_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self::with_declared_graph_metadata_dependencies_and_resources(
            pass_name,
            executor_id,
            queue,
            declared_queue,
            flags,
            Vec::new(),
            resources,
        )
    }

    pub fn with_declared_graph_metadata_dependencies_and_resources(
        pass_name: impl Into<String>,
        executor_id: RenderPassExecutorId,
        queue: QueueLane,
        declared_queue: QueueLane,
        flags: PassFlags,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        Self {
            pass_name: pass_name.into(),
            executor_id,
            declared_queue,
            queue,
            flags,
            dependencies,
            resources,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn uses_queue_fallback(&self) -> bool {
        self.declared_queue != self.queue
    }
}
