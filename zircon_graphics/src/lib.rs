//! Core rendering, scene rasterization, and host-agnostic GPU services.

mod backend;
mod extract;
mod feature;
mod material;
mod pipeline;
mod runtime;
mod scene;
mod shader;
mod types;
mod visibility;

pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{BuiltinRenderFeature, RenderFeature, RenderFeatureDescriptor};
pub use material::MaterialDomain;
pub use pipeline::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererAsset, RendererFeatureAsset,
};
pub use runtime::{offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings, WgpuRenderFramework};
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{SceneRenderer, ViewportIconSource};
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub use types::{
    EditorOrRuntimeFrame, GpuResourceHandle, GraphicsError, ViewportFrame,
    ViewportFrameTextureHandle,
};
pub use visibility::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBounds, VisibilityBvhInstance,
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityContext, VisibilityDrawCommand,
    VisibilityHistoryEntry, VisibilityHistorySnapshot, VisibilityHybridGiFeedback,
    VisibilityHybridGiProbe, VisibilityHybridGiUpdatePlan, VisibilityInstanceUploadPlan,
    VisibilityParticleUploadPlan, VisibilityVirtualGeometryCluster,
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

#[cfg(test)]
mod tests;
