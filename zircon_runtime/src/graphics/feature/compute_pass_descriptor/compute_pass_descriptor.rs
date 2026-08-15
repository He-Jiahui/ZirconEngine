use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    BindingSchemaEntry, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
    RenderGraphComputePassMetadata,
};

use super::ComputeShaderSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputePassDescriptor {
    pub pass_name: String,
    pub stage: RenderPassStage,
    pub queue: QueueLane,
    pub shader: ComputeShaderSource,
    pub entry_point: String,
    pub workgroup_size: [u32; 3],
    pub bindings: Vec<BindingSchemaEntry>,
    pub dispatch: RenderGraphComputeDispatchExtent,
    pub flags: PassFlags,
}

impl ComputePassDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pass_name: impl Into<String>,
        stage: RenderPassStage,
        queue: QueueLane,
        shader: ComputeShaderSource,
        entry_point: impl Into<String>,
        workgroup_size: [u32; 3],
        bindings: Vec<BindingSchemaEntry>,
        dispatch: RenderGraphComputeDispatchExtent,
        flags: PassFlags,
    ) -> Self {
        Self {
            pass_name: pass_name.into(),
            stage,
            queue,
            shader,
            entry_point: entry_point.into(),
            workgroup_size,
            bindings,
            dispatch,
            flags,
        }
    }

    pub(crate) fn graph_metadata(&self) -> RenderGraphComputePassMetadata {
        RenderGraphComputePassMetadata::new(
            self.shader.graph_source(),
            self.entry_point.clone(),
            self.bindings.clone(),
        )
    }
}
