//! Core rendering, scene rasterization, and host-agnostic GPU services.

// Crate-private implementation owners. Public callers should enter graphics
// through the curated facade exports and subsystem prelude below.
pub(crate) mod backend;
pub(crate) mod debug_markers;
pub(crate) mod extract;
pub(crate) mod feature;
pub(crate) mod hybrid_gi_runtime_provider;
pub(crate) mod material;
pub(crate) mod particle_runtime_provider;
pub(crate) mod pipeline;
pub(crate) mod resource_limits;
pub(crate) mod runtime;
mod runtime_prepare_collector;
pub(crate) mod runtime_provider;
pub(crate) mod scene;
pub(crate) mod shader;
pub(crate) mod solari_runtime_provider;
pub(crate) mod text;
pub(crate) mod types;
pub(crate) mod virtual_geometry_runtime_provider;
pub(crate) mod visibility;

// Public module entries: feature-specific extract source contracts, the common
// prelude, and the module descriptor surface.
pub mod hybrid_gi_extract_sources;
pub mod prelude;
pub mod runtime_builtin_graphics;

// Public facade exports. These are intentionally grouped by owner module so the
// facade remains reviewable while implementation modules stay crate-private.
pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{
    BuiltinRenderFeature, RenderFeature, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
pub use hybrid_gi_runtime_provider::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput,
    HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration,
    HybridGiRuntimeState, HybridGiRuntimeStats, HybridGiRuntimeUpdate,
};
pub use material::MaterialDomain;
pub use particle_runtime_provider::{ParticleGpuFeedback, ParticleRuntimeFeedback};
pub use pipeline::{
    CompiledRenderPipeline, CompiledRenderPipelinePassStage, RenderPassStage, RenderPipelineAsset,
    RenderPipelineAssetContext, RenderPipelineCompileOptions, RenderPipelineCompileReport,
    RendererAsset, RendererDataDocument, RendererDataDocumentError, RendererFeatureAsset,
    RendererFeatureAssetReferences, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity, RendererFeatureDocument,
    RendererFeatureReferenceListKind, RendererFeatureSource, RENDERER_DATA_DOCUMENT_VERSION,
};
pub use runtime::{
    offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings, WgpuRenderFramework,
};
pub use runtime_builtin_graphics::{
    module_descriptor as graphics_module_descriptor, GraphicsModule, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};

// Crate-visible bridge used by runtime preparation paths without widening the
// public graphics API.
pub(crate) use runtime_prepare_collector::RuntimePrepareExternalBufferBinding;
pub use runtime_prepare_collector::{
    RuntimePrepareCollector, RuntimePrepareCollectorContext, RuntimePrepareCollectorFn,
    RuntimePrepareCollectorRegistration,
};

// Test-only access for graphics surface assertions.
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{
    ParticleGpuTransparentDrawContext, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext, SceneRenderer,
};
pub use shader::{MaterialGraphAsset, ShaderGraphAsset, ShaderProgramAsset, ShaderVariantKey};
pub use solari_runtime_provider::{SolariRuntimeProvider, SolariRuntimeProviderRegistration};
pub use types::{
    GpuResourceHandle, GraphicsError, ViewportFrame, ViewportFrameTextureHandle,
    ViewportRenderRegion,
};
pub(crate) use types::{
    ViewportCameraStackOutputPolicy, ViewportRenderFrame, ViewportRenderOutputTarget,
};
pub use virtual_geometry_runtime_provider::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeExtractOutput,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeProviderRegistration, VirtualGeometryRuntimeState,
    VirtualGeometryRuntimeStats, VirtualGeometryRuntimeUpdate,
};
pub use visibility::{
    FrameVisibility, HzbBuildPlan, HzbBuilder, ViewCullingStats, ViewVisibilityContext,
    VisibilityBatch, VisibilityBatchKey, VisibilityBounds, VisibilityBvhInstance,
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityContext, VisibilityDrawCommand,
    VisibilityHistoryEntry, VisibilityHistorySnapshot, VisibilityHybridGiFeedback,
    VisibilityHybridGiProbe, VisibilityHybridGiUpdatePlan, VisibilityInstanceUploadPlan,
    VisibilityParticleUploadPlan, VisibilityStaticIndexReport, VisibilityViewKey,
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

#[cfg(test)]
mod tests;
