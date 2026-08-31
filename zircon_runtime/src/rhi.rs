//! Curated runtime facade for backend-neutral RHI contracts.

pub use zr_rhi::{
    AccelerationStructureCaps, AddressMode, BindGroupDesc, BindGroupEntryDesc,
    BindGroupEntryResource, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutEntryDesc,
    BindGroupLayoutHandle, BindingResourceType, BlendComponentDesc, BlendFactor, BlendOperation,
    BlendStateDesc, BufferDesc, BufferHandle, BufferUsage, ColorTargetDesc, ColorWriteMask,
    CommandList, CommandListCommand, CompareFunction, CullMode, DepthStencilStateDesc,
    DeviceGeneration, DeviceId, FilterMode, FrontFace, GpuMemoryBudget, GpuMemoryClass,
    GpuMemorySnapshot, IndexFormat, MipmapFilterMode, PipelineDesc, PipelineHandle, PipelineKind,
    PipelineLayoutDesc, PipelineLayoutHandle, PresentMode, PrimitiveStateDesc, PrimitiveTopology,
    RasterPipelineStateDesc, RenderAdapterInfo, RenderBackendCaps, RenderClearColor,
    RenderDebugInstrumentationStatus, RenderDevice, RenderDeviceLimits, RenderDeviceProfile,
    RenderNativeSurfaceTarget, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp,
    RenderPassStoreOp, RenderPassTextureViewDesc, RenderQueueClass, RenderScissorRect,
    RenderSurfaceDescriptor, RenderViewportDesc, RhiError, RhiSubmissionPacket, SamplerBindingType,
    SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, ShaderStage,
    StorageTextureAccess, StorageTextureBindingDesc, SubmissionHistory, SubmissionLimits,
    SubmissionPollReceipt, SubmissionStatus, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceRetryReason, SurfaceSessionCreateOutcome, SurfaceSessionReceipt, SwapchainDesc,
    TextureCopyAspect, TextureCopyRegion, TextureDesc, TextureDimension, TextureFormat,
    TextureHandle, TextureResidency, TextureSampleType, TextureUsage, TextureViewAspect,
    TextureViewDesc, TextureViewDimension, TextureViewHandle, TransientAllocatorStats,
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList,
    UiSurfaceImagePayload, UiSurfaceImageResource, UiSurfaceImageResourceTable,
    UiSurfaceImageUvRect, UiSurfacePresentOutcome, UiSurfacePresentStats, UiSurfacePresenter,
    UiSurfaceRect, UiSurfaceResolvedCommandKind, UiSurfaceStyle, UiSurfaceStyleHandle,
    UiSurfaceStyledPayload, UiSurfaceTextStyle, VertexAttributeDesc, VertexBufferLayoutDesc,
    VertexFormat, VertexInputLayoutDesc, VertexStepMode,
};

// Submission-qualified diagnostic readback contracts.
pub use zr_rhi::{
    DiagnosticFrameKey, DiagnosticReadbackAdmission, DiagnosticReadbackBudget,
    DiagnosticReadbackError, DiagnosticReadbackKind, DiagnosticReadbackReceipt,
    DiagnosticReadbackRequestId, DiagnosticReadbackTerminal, DiagnosticReadbackTracker,
};

pub fn create_default_ui_surface_presenter(
    descriptor: UiSurfaceDescriptor,
) -> Result<Box<dyn UiSurfacePresenter>, RhiError> {
    Ok(Box::new(zr_rhi_wgpu::WgpuUiSurfacePresenter::new(
        descriptor,
    )?))
}
