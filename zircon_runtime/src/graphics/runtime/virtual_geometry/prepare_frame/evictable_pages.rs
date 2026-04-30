use crate::graphics::types::VirtualGeometryPreparePage;

use super::super::VirtualGeometryRuntimeState;

pub(super) fn evictable_pages(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPreparePage> {
    state
        .evictable_page_ids()
        .iter()
        .filter_map(|page_id| {
            state
                .resident_slot(*page_id)
                .map(|slot| VirtualGeometryPreparePage {
                    page_id: *page_id,
                    slot,
                    size_bytes: state.page_size_bytes(*page_id),
                })
        })
        .collect::<Vec<_>>()
}
