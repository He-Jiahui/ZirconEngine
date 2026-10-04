//! High-frequency graphics imports for runtime render setup and integration code.

pub use super::{
    graphics_module_descriptor, BuiltinRenderFeature, CompiledRenderPipeline, FrameHistoryAccess,
    FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot, GpuResourceHandle, GraphicsError,
    GraphicsModule, OfflineBakeOutput, OfflineBakeSettings, RenderBufferSchema, RenderFeature,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassExecutor, RenderPassExecutorId,
    RenderPassExecutorRegistration, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RenderPipelineCompileReport, RenderResourceFallback,
    RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureExtentReference,
    RenderTextureExtentRounding, RenderTextureSchema, RuntimeGpuReadback, RuntimePrepareCollector,
    RuntimePrepareCollectorContext, RuntimePrepareCollectorFn, RuntimePrepareCollectorRegistration,
    RuntimePrepareMaterialCaptureSeed, RuntimePrepareMeshGeometrySeed,
    RuntimePrepareMeshSdfDeformationReason, RuntimePrepareMeshSdfSeed, SceneRenderer,
    ViewportFrame, ViewportFrameTextureHandle, ViewportRenderRegion, WgpuRenderFramework,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
};
