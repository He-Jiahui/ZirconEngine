//! Render graph construction and compilation.

mod builder;
mod dump;
mod error;
mod graph;
mod store_lint;
mod types;

pub use builder::RenderGraphBuilder;
pub use dump::{
    RenderGraphDump, RenderGraphDumpPassResourceRow, RenderGraphDumpPassRow,
    RenderGraphDumpResourceDesc, RenderGraphDumpResourceRow, RenderGraphDumpTransientSlotRow,
};
pub use error::RenderGraphError;
pub use graph::{
    CompiledRenderGraph, CompiledRenderGraphStats, CompiledRenderGraphTransientAllocation,
    CompiledRenderGraphTransientAllocationPlan, CompiledRenderPass,
};
pub use store_lint::{
    RenderGraphAttachmentBandwidthLedger, RenderGraphAttachmentBandwidthRow,
    RenderGraphStoreLintKind, RenderGraphStoreLintReport, RenderGraphStoreLintRow,
};
pub use types::{
    BindingSchemaEntry, ComputeBindingKind, ExternalResource, PassFlags, QueueLane,
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphExternalResourceRequirement, RenderGraphExternalResourceType,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderGraphResourceUsageFlags, RenderGraphResourceVersion,
    RenderPassId, RgBufferHandle, RgTextureHandle,
};

#[cfg(test)]
mod tests;
