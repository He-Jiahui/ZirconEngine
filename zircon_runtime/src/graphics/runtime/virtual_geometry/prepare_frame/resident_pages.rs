use crate::graphics::types::VirtualGeometryPreparePage;

use super::super::VirtualGeometryRuntimeState;

pub(super) fn resident_pages(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPreparePage> {
    state
        .resident_page_slots()
        .map(|(page_id, slot)| VirtualGeometryPreparePage {
            page_id,
            slot,
            size_bytes: state.page_size_bytes(page_id),
        })
        .collect::<Vec<_>>()
}
