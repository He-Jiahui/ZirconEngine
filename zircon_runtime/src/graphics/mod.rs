//! Core rendering, scene rasterization, and host-agnostic GPU services.

pub(crate) mod backend;
pub(crate) mod extract;
pub(crate) mod feature;
pub(crate) mod material;
pub(crate) mod pipeline;
pub(crate) mod runtime;
pub(crate) mod scene;
pub(crate) mod shader;
pub(crate) mod types;
pub(crate) mod visibility;

pub mod runtime_builtin_graphics;

pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{
    BuiltinRenderFeature, RenderFeature, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceKind,
};
pub use material::MaterialDomain;
pub use pipeline::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererAsset, RendererFeatureAsset, RendererFeatureSource,
};
pub use runtime::{
    offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings, WgpuRenderFramework,
};
pub use runtime_builtin_graphics::{
    module_descriptor as graphics_module_descriptor, GraphicsModule, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};
pub use scene::SceneRenderer;
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub(crate) use types::ViewportRenderFrame;
pub use types::{GpuResourceHandle, GraphicsError, ViewportFrame, ViewportFrameTextureHandle};
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
