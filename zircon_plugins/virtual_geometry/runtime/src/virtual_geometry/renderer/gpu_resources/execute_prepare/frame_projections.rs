use crate::virtual_geometry::types::VirtualGeometryPrepareFrame;

pub(super) fn resident_entries_and_slots(
    prepare: &VirtualGeometryPrepareFrame,
) -> (Vec<[u32; 2]>, Vec<u32>) {
    let mut resident_entries = Vec::with_capacity(prepare.resident_pages.len());
    let mut resident_slots = Vec::with_capacity(prepare.resident_pages.len());
    for page in &prepare.resident_pages {
        resident_entries.push([page.page_id, page.slot]);
        resident_slots.push(page.slot);
    }
    (resident_entries, resident_slots)
}

pub(super) fn evictable_slots_and_reclaimable_bytes(
    prepare: &VirtualGeometryPrepareFrame,
) -> (Vec<u32>, u32) {
    let mut evictable_slots = Vec::with_capacity(prepare.evictable_pages.len());
    let mut reclaimable_bytes = 0_u64;
    for page in &prepare.evictable_pages {
        evictable_slots.push(page.slot);
        reclaimable_bytes = reclaimable_bytes.saturating_add(page.size_bytes);
    }
    (
        evictable_slots,
        reclaimable_bytes.min(u64::from(u32::MAX)) as u32,
    )
}

#[cfg(test)]
mod performance_tests;
