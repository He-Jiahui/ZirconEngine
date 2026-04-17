use crate::types::VirtualGeometryPrepareRequest;

use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

pub(super) fn pending_page_requests(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPrepareRequest> {
    state
        .pending_requests
        .iter()
        .map(|request| VirtualGeometryPrepareRequest {
            page_id: request.page_id,
            size_bytes: request.size_bytes,
            generation: request.generation,
        })
        .collect::<Vec<_>>()
}
