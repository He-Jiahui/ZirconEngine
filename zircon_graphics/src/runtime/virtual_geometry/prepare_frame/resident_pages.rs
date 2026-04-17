use crate::types::VirtualGeometryPreparePage;

use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

pub(super) fn resident_pages(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPreparePage> {
    state
        .resident_slots
        .iter()
        .map(|(&page_id, &slot)| VirtualGeometryPreparePage {
            page_id,
            slot,
            size_bytes: state.page_sizes.get(&page_id).copied().unwrap_or_default(),
        })
        .collect::<Vec<_>>()
}
