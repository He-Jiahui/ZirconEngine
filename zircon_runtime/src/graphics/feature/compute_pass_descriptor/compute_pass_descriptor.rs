use std::collections::BTreeMap;

use crate::graphics::feature::RenderResourceSchema;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    BindingSchemaEntry, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
    RenderGraphComputePassMetadata, RenderGraphComputePipelineFallbackPolicy,
};

use super::ComputeShaderSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputePassDescriptor {
    pub pass_name: String,
    pub stage: RenderPassStage,
    pub queue: QueueLane,
    pub shader: ComputeShaderSource,
    pub pipeline_fallback_policy: RenderGraphComputePipelineFallbackPolicy,
    pub entry_point: String,
    pub workgroup_size: [u32; 3],
    pub bindings: Vec<BindingSchemaEntry>,
    pub dispatch: RenderGraphComputeDispatchExtent,
    pub flags: PassFlags,
    resource_schemas: BTreeMap<String, RenderResourceSchema>,
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
            pipeline_fallback_policy: RenderGraphComputePipelineFallbackPolicy::Reject,
            entry_point: entry_point.into(),
            workgroup_size,
            bindings,
            dispatch,
            flags,
            resource_schemas: BTreeMap::new(),
        }
    }

    pub fn with_last_good_pipeline(
        mut self,
        family: impl Into<String>,
        interface_generation: u64,
    ) -> Self {
        self.pipeline_fallback_policy =
            RenderGraphComputePipelineFallbackPolicy::last_good(family, interface_generation);
        self
    }

    /// Supplies the physical contract for a named graph resource used by
    /// this compute pass. Storage texture outputs never infer a format from
    /// their debug name.
    pub fn with_resource_schema(
        mut self,
        resource: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.resource_schemas.insert(resource.into(), schema);
        self
    }

    pub(crate) fn resource_schema(&self, resource: &str) -> Option<RenderResourceSchema> {
        self.resource_schemas.get(resource).copied()
    }

    pub(crate) fn graph_metadata(&self) -> RenderGraphComputePassMetadata {
        RenderGraphComputePassMetadata::new(
            self.shader.graph_source(),
            self.entry_point.clone(),
            self.bindings.clone(),
        )
    }
}
