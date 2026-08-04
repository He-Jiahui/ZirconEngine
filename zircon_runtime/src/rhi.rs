//! Curated runtime facade for backend-neutral RHI contracts.

pub use zr_rhi::{
    AccelerationStructureCaps, AddressMode, BindGroupDesc, BindGroupEntryDesc,
    BindGroupEntryResource, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutEntryDesc,
    BindGroupLayoutHandle, BindingResourceType, BlendComponentDesc, BlendFactor, BlendOperation,
    BlendStateDesc, BufferDesc, BufferHandle, BufferUsage, ColorTargetDesc, ColorWriteMask,
    CommandList, CommandListCommand, CompareFunction, CullMode, DepthStencilStateDesc, FenceValue,
    FilterMode, FrontFace, IndexFormat, MipmapFilterMode, PipelineDesc, PipelineHandle,
    PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, PresentMode, PrimitiveStateDesc,
    PrimitiveTopology, RasterPipelineStateDesc, RenderAdapterInfo, RenderBackendCaps,
    RenderClearColor, RenderDebugInstrumentationStatus, RenderDevice, RenderDeviceLimits,
    RenderNativeSurfaceTarget, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp,
    RenderPassStoreOp, RenderPassTextureViewDesc, RenderQueueClass, RenderScissorRect,
    RenderViewportDesc, RhiError, SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle,
    ShaderStage, SwapchainDesc, TextureCopyRegion, TextureDesc, TextureDimension, TextureFormat,
    TextureHandle, TextureResidency, TextureUsage, TransientAllocatorStats, UiSurfaceCommand,
    UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageResource, UiSurfaceImageResourceTable, UiSurfaceImageUvRect,
    UiSurfacePresentStats, UiSurfacePresenter, UiSurfaceRect, UiSurfaceResolvedCommandKind,
    UiSurfaceStyle, UiSurfaceStyleHandle, UiSurfaceStyledPayload, UiSurfaceTextStyle,
    VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc,
    VertexStepMode,
};

pub fn create_default_ui_surface_presenter(
    descriptor: UiSurfaceDescriptor,
) -> Result<Box<dyn UiSurfacePresenter>, RhiError> {
    Ok(Box::new(zr_rhi_wgpu::WgpuUiSurfacePresenter::new(
        descriptor,
    )?))
}
