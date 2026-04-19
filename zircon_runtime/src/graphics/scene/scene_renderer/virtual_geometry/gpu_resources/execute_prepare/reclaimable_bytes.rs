use crate::graphics::types::VirtualGeometryPrepareFrame;

pub(super) fn reclaimable_bytes(prepare: &VirtualGeometryPrepareFrame) -> u32 {
    prepare
        .evictable_pages
        .iter()
        .fold(0_u64, |bytes, page| bytes.saturating_add(page.size_bytes))
        .min(u64::from(u32::MAX)) as u32
}
