use zr_rhi::{BindGroupEntryResource, RhiError};

use super::registry::WgpuResourceRegistry;

/// Converts the admitted M7 typed binding subset at the native boundary.
pub(super) fn wgpu_bind_group_entry(
    registry: &WgpuResourceRegistry,
    resource: BindGroupEntryResource,
    binding: u32,
) -> Result<wgpu::BindGroupEntry<'_>, RhiError> {
    let resource = match resource {
        BindGroupEntryResource::Buffer(binding) => {
            wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: registry.buffer(binding.buffer)?,
                offset: binding.offset,
                size: binding.size.and_then(std::num::NonZeroU64::new),
            })
        }
        BindGroupEntryResource::TextureView(handle) => {
            wgpu::BindingResource::TextureView(registry.texture_view(handle)?)
        }
        BindGroupEntryResource::Sampler(handle) => {
            wgpu::BindingResource::Sampler(registry.sampler(handle)?)
        }
    };
    Ok(wgpu::BindGroupEntry { binding, resource })
}
