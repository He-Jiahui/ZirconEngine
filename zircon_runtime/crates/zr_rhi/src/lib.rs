//! Render hardware interface contracts and descriptors.

mod capabilities;
mod descriptors;
mod device;
mod native_surface;
mod ui_surface;

pub use capabilities::{
    AccelerationStructureCaps, RenderAdapterInfo, RenderBackendCaps,
    RenderDebugInstrumentationStatus, RenderDeviceLimits, RenderQueueClass,
};
pub use descriptors::{
    AddressMode, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingResourceType,
    BlendComponentDesc, BlendFactor, BlendOperation, BlendStateDesc, BufferDesc, BufferUsage,
    ColorTargetDesc, ColorWriteMask, CompareFunction, CullMode, DepthStencilStateDesc, FilterMode,
    FrontFace, MipmapFilterMode, PipelineDesc, PipelineKind, PipelineLayoutDesc, PresentMode,
    PrimitiveStateDesc, PrimitiveTopology, RasterPipelineStateDesc, SamplerDesc, ShaderModuleDesc,
    ShaderStage, SwapchainDesc, TextureDesc, TextureDimension, TextureFormat, TextureResidency,
    TextureUsage, VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc,
    VertexStepMode,
};
pub use device::{
    BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource, BindGroupHandle,
    BindGroupLayoutHandle, BufferHandle, CommandList, CommandListCommand, FenceValue, IndexFormat,
    PipelineHandle, PipelineLayoutHandle, RenderClearColor, RenderDevice,
    RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp, RenderPassStoreOp,
    RenderPassTextureViewDesc, RenderScissorRect, RenderViewportDesc, RhiError, SamplerHandle,
    ShaderModuleHandle, TextureCopyRegion, TextureHandle, TransientAllocatorStats,
};
pub use native_surface::RenderNativeSurfaceTarget;
pub use ui_surface::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList,
    UiSurfaceImagePayload, UiSurfaceImageResource, UiSurfaceImageResourceTable,
    UiSurfaceImageUvRect, UiSurfacePresentStats, UiSurfacePresentStatsAccumulator,
    UiSurfacePresenter, UiSurfaceRect, UiSurfaceResolvedCommandKind, UiSurfaceStyle,
    UiSurfaceStyleHandle, UiSurfaceStyledPayload, UiSurfaceTextStyle,
};

#[cfg(test)]
mod tests;
