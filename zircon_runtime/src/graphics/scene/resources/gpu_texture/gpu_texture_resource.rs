use core::ops::Range;
use std::sync::Arc;

use super::TextureSamplerCache;

use crate::core::framework::render::RenderImageDescriptor;
use crate::core::resource::ResourceId;

pub(crate) struct GpuTextureResource {
    pub(crate) id: Option<ResourceId>,
    pub(crate) descriptor: RenderImageDescriptor,
    pub(in crate::graphics::scene::resources) texture: wgpu::Texture,
    pub(in crate::graphics::scene::resources) view: wgpu::TextureView,
    pub(in crate::graphics::scene::resources) sampler: Arc<wgpu::Sampler>,
    pub(in crate::graphics::scene::resources) sampler_cache: Arc<TextureSamplerCache>,
    pub(in crate::graphics::scene::resources) mip_streaming_supported: bool,
    /// Logical byte size of the physical mip chain created for this resource.
    pub(in crate::graphics::scene::resources) resident_texture_bytes: u64,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl GpuTextureResource {
    pub(crate) const RETAINED_TEXTURE_BINDING_OWNER_COUNT: usize = 5;

    pub(crate) fn retained_texture_binding_owner_count(&self) -> usize {
        let _retained_texture_binding_owners = (
            &self.id,
            &self.texture,
            &self.view,
            &self.sampler,
            &self.sampler_cache,
        );
        Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        debug_assert_eq!(
            self.retained_texture_binding_owner_count(),
            Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT,
            "GpuTextureResource must retain identity, texture, view, sampler, and sampler-cache lifetime while exposing bindings",
        );
        &self.view
    }

    pub(crate) fn sampler(&self) -> &wgpu::Sampler {
        debug_assert_eq!(
            self.retained_texture_binding_owner_count(),
            Self::RETAINED_TEXTURE_BINDING_OWNER_COUNT,
            "GpuTextureResource must retain identity, texture, view, sampler, and sampler-cache lifetime while exposing bindings",
        );
        self.sampler.as_ref()
    }

    pub(in crate::graphics::scene::resources) const fn supports_mip_streaming(&self) -> bool {
        self.mip_streaming_supported
    }

    pub(in crate::graphics::scene::resources) const fn resident_texture_bytes(&self) -> u64 {
        self.resident_texture_bytes
    }

    /// Returns the exact source upload bytes for a target range after shared resident mips are
    /// copied directly between GPU textures. Only the RGBA8 physical-streaming path calls this.
    pub(in crate::graphics::scene::resources) fn mip_streaming_upload_bytes(
        &self,
        resident_mips: Range<u8>,
        wanted_mips: Range<u8>,
    ) -> u64 {
        if !self.mip_streaming_supported {
            return 0;
        }
        let mip_count = self.descriptor.mip_count.clamp(1, u32::from(u8::MAX)) as u8;
        rgba8_mip_streaming_upload_bytes(
            self.descriptor.width,
            self.descriptor.height,
            self.descriptor.depth_or_array_layers.max(1),
            mip_count,
            resident_mips,
            wanted_mips,
        )
    }

    pub(in crate::graphics::scene::resources) fn mip_streaming_resident_bytes(
        &self,
        resident_mips: Range<u8>,
    ) -> u64 {
        if !self.mip_streaming_supported {
            return self.resident_texture_bytes;
        }
        let mip_count = self.descriptor.mip_count.clamp(1, u32::from(u8::MAX)) as u8;
        Self::rgba8_mip_chain_bytes(
            self.descriptor.width,
            self.descriptor.height,
            self.descriptor.depth_or_array_layers.max(1),
            u32::from(resident_mips.start.min(mip_count.saturating_sub(1)))
                ..u32::from(resident_mips.end.min(mip_count)),
        )
    }

    pub(in crate::graphics::scene::resources) fn rgba8_mip_chain_bytes(
        width: u32,
        height: u32,
        layer_count: u32,
        mip_range: Range<u32>,
    ) -> u64 {
        let layer_count = u64::from(layer_count.max(1));
        mip_range
            .map(|level| {
                u64::from(mip_extent(width, level))
                    .saturating_mul(u64::from(mip_extent(height, level)))
                    .saturating_mul(layer_count)
                    .saturating_mul(4)
            })
            .fold(0_u64, u64::saturating_add)
    }
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

fn rgba8_mip_streaming_upload_bytes(
    width: u32,
    height: u32,
    layer_count: u32,
    mip_count: u8,
    resident_mips: Range<u8>,
    wanted_mips: Range<u8>,
) -> u64 {
    let wanted_start = wanted_mips.start.min(mip_count.saturating_sub(1));
    let wanted_end = wanted_mips.end.min(mip_count);
    let layer_count = u64::from(layer_count.max(1));
    (wanted_start..wanted_end)
        .filter(|level| !resident_mips.contains(level))
        .map(|level| {
            let width = mip_extent(width, u32::from(level));
            let height = mip_extent(height, u32::from(level));
            u64::from(width)
                .saturating_mul(u64::from(height))
                .saturating_mul(layer_count)
                .saturating_mul(4)
        })
        .fold(0_u64, u64::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::{rgba8_mip_streaming_upload_bytes, GpuTextureResource};

    #[test]
    fn mip_streaming_upload_bytes_excludes_mips_copied_from_prior_residency() {
        assert_eq!(
            rgba8_mip_streaming_upload_bytes(8, 4, 2, 4, 2..4, 0..4),
            320,
            "only mip zero and one are reuploaded while the resident tail is copied on-GPU"
        );
        assert_eq!(
            rgba8_mip_streaming_upload_bytes(8, 4, 2, 4, 0..4, 2..4),
            0,
            "eviction recreates the physical tail solely through GPU copies"
        );
    }

    #[test]
    fn mip_streaming_resident_bytes_tracks_the_physical_tail_range() {
        assert_eq!(
            GpuTextureResource::rgba8_mip_chain_bytes(8, 4, 2, 2..4),
            40,
            "the physical tail contains only source levels two and three"
        );
        assert_eq!(
            GpuTextureResource::rgba8_mip_chain_bytes(8, 4, 2, 0..4),
            340,
            "a fully resident chain accounts for every source mip"
        );
    }
}
