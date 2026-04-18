use crate::types::VirtualGeometryPreparePage;

use super::super::VirtualGeometryRuntimeState;

pub(super) fn evictable_pages(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPreparePage> {
    state
        .evictable_pages
        .iter()
        .filter_map(|page_id| {
            state
                .resident_slots
                .get(page_id)
                .copied()
                .map(|slot| VirtualGeometryPreparePage {
                    page_id: *page_id,
                    slot,
                    size_bytes: state.page_sizes.get(page_id).copied().unwrap_or_default(),
                })
        })
        .collect::<Vec<_>>()
}
