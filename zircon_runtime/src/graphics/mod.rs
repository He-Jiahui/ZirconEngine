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
pub use feature::{BuiltinRenderFeature, RenderFeature, RenderFeatureDescriptor};
pub use material::MaterialDomain;
pub use pipeline::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererAsset, RendererFeatureAsset,
};
pub use runtime::{
    offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings, WgpuRenderFramework,
};
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{SceneRenderer, ViewportIconSource};
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub use types::{
    EditorOrRuntimeFrame, GpuResourceHandle, GraphicsError, ViewportFrame,
    ViewportFrameTextureHandle,
};
pub use runtime_builtin_graphics::{
    module_descriptor as graphics_module_descriptor, GraphicsModule, GRAPHICS_MODULE_NAME,
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME,
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
