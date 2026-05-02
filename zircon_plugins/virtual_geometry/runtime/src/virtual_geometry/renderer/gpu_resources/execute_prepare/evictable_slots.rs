use crate::virtual_geometry::types::VirtualGeometryPrepareFrame;

pub(super) fn evictable_slots(prepare: &VirtualGeometryPrepareFrame) -> Vec<u32> {
    prepare
        .evictable_pages
        .iter()
        .map(|page| page.slot)
        .collect()
}
