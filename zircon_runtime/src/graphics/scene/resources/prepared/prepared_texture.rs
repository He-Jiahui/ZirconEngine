use core::ops::Range;
use std::sync::Arc;

use super::super::GpuTextureResource;

pub(in crate::graphics::scene::resources) struct PreparedTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuTextureResource>,
    /// The contiguous source mip range currently represented by `resource`.
    pub(in crate::graphics::scene::resources) resident_mip_range: Range<u8>,
}

impl PreparedTexture {
    pub(in crate::graphics::scene::resources) fn fully_resident(
        revision: u64,
        resource: Arc<GpuTextureResource>,
    ) -> Self {
        let mip_count = resource.descriptor.mip_count.clamp(1, u32::from(u8::MAX)) as u8;
        Self {
            revision,
            resource,
            resident_mip_range: 0..mip_count,
        }
    }
}
