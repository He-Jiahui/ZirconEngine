//! High-frequency graphics imports for runtime render setup and integration code.

pub use super::{
    BuiltinRenderFeature, CompiledRenderPipeline, FrameHistoryAccess, FrameHistoryBinding,
    FrameHistoryHandle, FrameHistorySlot, GpuResourceHandle, GraphicsError, GraphicsModule,
    OfflineBakeOutput, OfflineBakeSettings, RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME,
    RenderBufferSchema, RenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceWriteMode, RenderPassExecutor,
    RenderPassExecutorId, RenderPassExecutorRegistration, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RenderPipelineCompileReport, RenderResourceFallback,
    RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureExtentReference,
    RenderTextureExtentRounding, RenderTextureSchema, RuntimeGpuReadback, RuntimePrepareCollector,
    RuntimePrepareCollectorContext, RuntimePrepareCollectorFn, RuntimePrepareCollectorRegistration,
    RuntimePrepareMaterialCaptureSeed, RuntimePrepareMeshGeometrySeed,
    RuntimePrepareMeshSdfDeformationReason, RuntimePrepareMeshSdfSeed, SceneRenderer,
    ViewportFrame, ViewportFrameTextureHandle, ViewportRenderRegion, WgpuRenderFramework,
    graphics_module_descriptor,
};
