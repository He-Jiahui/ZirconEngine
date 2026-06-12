//! Render graph construction and compilation.

mod builder;
mod error;
mod graph;
mod types;

pub use builder::RenderGraphBuilder;
pub use error::RenderGraphError;
pub use graph::{
    CompiledRenderGraph, CompiledRenderGraphStats, CompiledRenderGraphTransientAllocation,
    CompiledRenderGraphTransientAllocationPlan, CompiledRenderPass,
};
pub use types::{
    ExternalResource, PassFlags, QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps,
    RenderGraphAttachmentStoreOp, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderPassId, RgBufferHandle, RgTextureHandle,
};

#[cfg(test)]
mod tests;
