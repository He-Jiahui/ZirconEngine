//! High-frequency graphics imports for runtime render setup and integration code.

pub use super::{
    BuiltinRenderFeature, CompiledRenderPipeline, CompiledRenderPipelinePassStage,
    FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot,
    GpuResourceHandle, GraphicsError, GraphicsModule, OfflineBakeOutput, OfflineBakeSettings,
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME, RenderFeature,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassExecutor, RenderPassExecutorId,
    RenderPassExecutorRegistration, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RenderPipelineCompileReport, RuntimePrepareCollector,
    RuntimePrepareCollectorContext, RuntimePrepareCollectorFn, RuntimePrepareCollectorRegistration,
    RuntimePrepareMaterialCaptureSeed, SceneRenderer, ViewportFrame, ViewportFrameTextureHandle,
    ViewportRenderRegion, WgpuRenderFramework, graphics_module_descriptor,
};
