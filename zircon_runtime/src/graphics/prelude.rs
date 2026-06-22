//! High-frequency graphics imports for runtime render setup and integration code.

pub use super::{
    graphics_module_descriptor, BuiltinRenderFeature, CompiledRenderPipeline,
    CompiledRenderPipelinePassStage, FrameHistoryAccess, FrameHistoryBinding, FrameHistoryHandle,
    FrameHistorySlot, GpuResourceHandle, GraphicsError, GraphicsModule, OfflineBakeOutput,
    OfflineBakeSettings, RenderFeature, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
    RenderPassExecutor, RenderPassExecutorId, RenderPassExecutorRegistration, RenderPassStage,
    RenderPipelineAsset, RenderPipelineCompileOptions, RenderPipelineCompileReport,
    RuntimePrepareCollector, RuntimePrepareCollectorContext, RuntimePrepareCollectorFn,
    RuntimePrepareCollectorRegistration, SceneRenderer, ViewportFrame, ViewportFrameTextureHandle,
    ViewportRenderRegion, WgpuRenderFramework, GRAPHICS_MODULE_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME,
};
