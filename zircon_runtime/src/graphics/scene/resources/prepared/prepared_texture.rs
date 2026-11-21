use core::ops::Range;
use std::sync::Arc;

use crate::core::resource::{
    ResourceActiveVersionKey, ResourceKind, ResourceManagementRowIdentity,
};

use super::super::GpuTextureResource;

/// 同一资产修订的设备纹理与当前连续 mip 驻留前沿；材质代理可在修订不变时换用更新的 mip 资源。
pub(in crate::graphics::scene::resources) struct PreparedTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    catalog_row: ResourceManagementRowIdentity,
    pub(in crate::graphics::scene::resources) active_key: ResourceActiveVersionKey,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuTextureResource>,
    /// Fixed CPU sample captured from the same installed payload as `resource`.
    pub(in crate::graphics::scene::resources) capture_sample_rgba: Option<[f32; 4]>,
    /// The contiguous source mip range currently represented by `resource`.
    pub(in crate::graphics::scene::resources) resident_mip_range: Range<u8>,
}

impl PreparedTexture {
    /// CPU eviction removes only the active key. A still-current catalog row can use the GPU copy.
    pub(in crate::graphics::scene::resources) fn matches_source(
        &self,
        catalog: &ResourceManagementRowIdentity,
        active: Option<&ResourceActiveVersionKey>,
    ) -> bool {
        catalog.row().kind == ResourceKind::Texture
            && catalog.row().revision == self.revision
            && active.map_or_else(
                || &self.catalog_row == catalog,
                |key| &self.active_key == key,
            )
    }

    pub(in crate::graphics::scene::resources) fn fully_resident(
        revision: u64,
        catalog_row: ResourceManagementRowIdentity,
        active_key: ResourceActiveVersionKey,
        resource: Arc<GpuTextureResource>,
        capture_sample_rgba: Option<[f32; 4]>,
    ) -> Self {
        let mip_count = resource.descriptor.mip_count.clamp(1, u32::from(u8::MAX)) as u8;
        Self {
            revision,
            catalog_row,
            active_key,
            resource,
            capture_sample_rgba,
            resident_mip_range: 0..mip_count,
        }
    }
}
