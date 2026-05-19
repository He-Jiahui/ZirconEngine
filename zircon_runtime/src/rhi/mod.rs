//! Render hardware interface contracts and descriptors.

mod capabilities;
mod descriptors;
mod device;
mod ui_surface;

pub use capabilities::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};
pub use descriptors::{
    AddressMode, BufferDesc, BufferUsage, PipelineDesc, PipelineKind, PresentMode, SamplerDesc,
    ShaderModuleDesc, ShaderStage, SwapchainDesc, TextureDesc, TextureDimension, TextureFormat,
    TextureUsage,
};
pub use device::{
    BufferHandle, CommandList, CommandListCommand, FenceValue, GpuBuffer, PipelineHandle,
    RenderDevice, RhiError, SamplerHandle, ShaderModuleHandle, TextureHandle,
    TransientAllocatorStats,
};
pub use ui_surface::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList,
    UiSurfaceImagePayload, UiSurfaceImageUvRect, UiSurfacePresentStats, UiSurfacePresenter,
    UiSurfaceRect, UiSurfaceTextStyle,
};

pub fn create_default_ui_surface_presenter(
    descriptor: UiSurfaceDescriptor,
) -> Result<Box<dyn UiSurfacePresenter>, RhiError> {
    Ok(Box::new(crate::rhi_wgpu::WgpuUiSurfacePresenter::new(
        descriptor,
    )?))
}

#[cfg(test)]
mod tests;
