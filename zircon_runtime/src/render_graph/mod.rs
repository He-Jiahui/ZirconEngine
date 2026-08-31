//! Render graph construction and compilation.

mod access;
mod builder;
mod dump;
mod error;
mod graph;
mod resource_schema;
mod store_lint;
mod types;

pub use access::{
    RenderGraphBufferRange, RenderGraphResourceAccessId, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureAspect, RenderGraphTextureSubresourceRange, RenderGraphVersionedAccessKey,
};
pub use builder::RenderGraphBuilder;
pub use dump::{
    RenderGraphDump, RenderGraphDumpPassResourceRow, RenderGraphDumpPassRow,
    RenderGraphDumpResourceDesc, RenderGraphDumpResourceRow, RenderGraphDumpTransientSlotRow,
};
pub use error::RenderGraphError;
pub use graph::{
    CompiledRenderGraph, CompiledRenderGraphAccessAllocationBinding,
    CompiledRenderGraphAccessAllocationTable, CompiledRenderGraphComputeBindingAccess,
    CompiledRenderGraphComputeBindingAccessPacket, CompiledRenderGraphComputeDispatchAccess,
    CompiledRenderGraphComputeDispatchAccessPacket, CompiledRenderGraphExternalAccess,
    CompiledRenderGraphExternalAccessPacket, CompiledRenderGraphStats,
    CompiledRenderGraphTransientAllocation, CompiledRenderGraphTransientAllocationId,
    CompiledRenderGraphTransientAllocationPlan, CompiledRenderGraphTransientSlotReservation,
    CompiledRenderPass, RenderGraphPhysicalAllocationId,
};
pub use resource_schema::{
    RenderBufferSchema, RenderResourceFallback, RenderResourceSchema, RenderTextureExtentPolicy,
    RenderTextureExtentReference, RenderTextureExtentRounding, RenderTextureSchema,
};
pub use store_lint::{
    RenderGraphAttachmentBandwidthLedger, RenderGraphAttachmentBandwidthRow,
    RenderGraphStoreLintKind, RenderGraphStoreLintReport, RenderGraphStoreLintRow,
};
pub use types::{
    BindingSchemaEntry, ComputeBindingKind, ExternalResource, PassFlags, QueueLane,
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphBufferBindingRange, RenderGraphComputeDispatchExtent,
    RenderGraphComputePassMetadata, RenderGraphComputePipelineFallbackPolicy,
    RenderGraphComputePipelineFamily, RenderGraphComputePipelineResolution,
    RenderGraphComputePipelineResolutionStatus, RenderGraphComputeShaderSource,
    RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphExternalResourceRequirement, RenderGraphExternalResourceType,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderGraphResourceUsageFlags, RenderGraphResourceVersion,
    RenderGraphResourceVersionToken, RenderGraphTextureViewAlias, RenderPassId, RgBufferHandle,
    RgTextureHandle,
};

#[cfg(test)]
mod tests;
