//! Render hardware interface contracts and descriptors.

mod capabilities;
mod descriptors;
mod device;

pub use capabilities::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};
pub use descriptors::{
    AddressMode, BufferDesc, BufferUsage, PipelineDesc, PipelineKind, PresentMode, SamplerDesc,
    ShaderModuleDesc, ShaderStage, SwapchainDesc, TextureDesc, TextureDimension, TextureFormat,
    TextureUsage,
};
pub use device::{
    BufferHandle, CommandList, FenceValue, PipelineHandle, RenderDevice, RhiError, SamplerHandle,
    ShaderModuleHandle, TextureHandle, TransientAllocatorStats,
};

#[cfg(test)]
mod tests;
