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
mod text_transport;
pub(crate) mod types;
pub(crate) mod virtual_geometry_runtime_provider;
pub(crate) mod visibility;

// Public module entries: the common prelude and module descriptor surface.
pub mod prelude;
pub mod runtime_builtin_graphics;

// Public facade exports. These are intentionally grouped by owner module so the
// facade remains reviewable while implementation modules stay crate-private.
pub use extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot};
pub use feature::{
    COMPUTE_GENERIC_EXECUTOR_ID, BuiltinRenderFeature, ComputePassDescriptor, ComputeShaderSource,
    RenderFeature, RenderFeatureCapabilityRequirement,
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
    CompiledRenderPipeline, CompiledRenderPipelinePassStage, RENDERER_DATA_DOCUMENT_VERSION,
    RenderPassStage, RenderPipelineAsset, RenderPipelineAssetContext, RenderPipelineCompileOptions,
    RenderPipelineCompileReport, RendererAsset, RendererDataDocument, RendererDataDocumentError,
    RendererFeatureAsset, RendererFeatureAssetReferences, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity, RendererFeatureDocument,
    RendererFeatureReferenceListKind, RendererFeatureSource,
};
pub use runtime::{
    OfflineBakeOutput, OfflineBakeSettings, WgpuRenderFramework, offline_bake_frame,
};
pub use runtime_builtin_graphics::{
    GraphicsModule, RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME,
    module_descriptor as graphics_module_descriptor,
};
// Crate-visible bridge used by runtime preparation paths without widening the
// public graphics API.
pub use runtime_prepare_collector::RuntimePrepareMaterialCaptureSeed;
pub use runtime_prepare_collector::{
    RuntimeGpuReadback, RuntimePrepareCollector, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorFn, RuntimePrepareCollectorRegistration,
};
pub(crate) use runtime_prepare_collector::{
    RuntimePrepareExternalBufferBinding, RuntimePrepareGpuReadbackRequest,
};

// Test-only access for graphics surface assertions.
#[cfg(test)]
pub(crate) use scene::ViewportOverlayRenderer;
pub use scene::{
    IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE,
    LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID, LIGHT_COOKIE_ATLAS_RESOURCE, OIT_DRAW_SHADER_SOURCE,
    OIT_FRAGMENT_STORE_EXECUTOR_ID, OIT_RESOLVE_EXECUTOR_ID, OIT_RESOLVE_SHADER_SOURCE,
    PLANAR_FILTER_EXECUTOR_ID, PLANAR_REFLECTION_TEXTURE_RESOURCE,
    ParticleGpuTransparentDrawContext, RealtimeIblGpuTimingReport, RenderGraphExecutionResources,
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorFn, RenderPassExecutorId,
    RenderPassExecutorRegistration, RenderPassGpuExecutionContext, RenderPassRecordingPolicy,
    RuntimeShaderPipelinePrewarmFailure, RuntimeShaderPipelinePrewarmReport,
    SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID, SceneRenderer,
    SceneRendererCoreStartupReport, SceneRendererDeferredLightingProfile,
    SceneRendererFrameTimingReport, SceneRendererStartupOptions, SceneRendererStartupReport,
    SceneViewportSurface, VOLUMETRIC_INTEGRATE_EXECUTOR_ID, VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID,
    VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID, irradiance_volume_render_pass_executor_registrations,
    light_cookie_render_pass_executor_registrations, oit_render_pass_executor_registrations,
    planar_reflection_filter_compute_workload,
    planar_reflection_render_pass_executor_registrations, subsurface_render_feature_descriptor,
    subsurface_render_pass_executor_registrations, subsurface_scatter_compute_workload,
    subsurface_setup_compute_workload, volumetric_fog_render_pass_executor_registrations,
};
pub use shader::{
    MaterialGraphAsset, ShaderGraphAsset, ShaderIdeEnvReport, ShaderIdePreviewError,
    ShaderIdePreviewVariant, ShaderIdeSurfacePreview, ShaderIdeWgslCheckError,
    ShaderIdeWgslModuleValidation, ShaderProgramAsset, ShaderVariantKey,
    assemble_shader_ide_surface_preview, builtin_shader_ide_module_sources,
    parse_shader_ide_wgsl_module, validate_shader_ide_wgsl_module,
    write_shader_ide_env_for_project,
};
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
