use crate::virtual_geometry::VirtualGeometryPreparePage;

use super::super::VirtualGeometryRuntimeState;

#[cfg(test)]
#[path = "evictable_pages/performance_tests.rs"]
mod performance_tests;

pub(super) fn evictable_pages(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPreparePage> {
    let evictable_page_ids = state.evictable_page_ids();
    let mut pages = Vec::with_capacity(evictable_page_ids.len());
    for &page_id in evictable_page_ids {
        let Some(slot) = state.resident_slot(page_id) else {
            continue;
        };
        pages.push(VirtualGeometryPreparePage {
            page_id,
            slot,
            size_bytes: state.page_size_bytes(page_id),
        });
    }
    pages
}
