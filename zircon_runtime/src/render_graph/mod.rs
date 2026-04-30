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
    ExternalResource, PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderPassId, TransientBuffer, TransientTexture,
};

#[cfg(test)]
mod tests;
