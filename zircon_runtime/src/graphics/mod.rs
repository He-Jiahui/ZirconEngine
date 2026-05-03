//! Core rendering, scene rasterization, and host-agnostic GPU services.

pub(crate) mod backend;
pub(crate) mod extract;
pub(crate) mod feature;
pub mod hybrid_gi_extract_sources;
pub(crate) mod hybrid_gi_runtime_provider;
pub(crate) mod material;
pub(crate) mod pipeline;
pub(crate) mod runtime;
pub(crate) mod scene;
pub(crate) mod shader;
pub(crate) mod types;
pub(crate) mod virtual_geometry_runtime_provider;
pub(crate) mod visibility;

pub mod runtime_builtin_graphics;

pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{
    BuiltinRenderFeature, RenderFeature, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceKind,
};
pub use hybrid_gi_runtime_provider::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput,
    HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration,
    HybridGiRuntimeState, HybridGiRuntimeStats, HybridGiRuntimeUpdate,
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
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{
    RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId,
    RenderPassExecutorRegistration, SceneRenderer,
};
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub(crate) use types::ViewportRenderFrame;
pub use types::{GpuResourceHandle, GraphicsError, ViewportFrame, ViewportFrameTextureHandle};
pub use virtual_geometry_runtime_provider::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeExtractOutput,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeStats, VirtualGeometryRuntimeUpdate,
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
