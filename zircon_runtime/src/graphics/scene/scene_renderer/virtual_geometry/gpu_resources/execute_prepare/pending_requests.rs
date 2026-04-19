use crate::graphics::types::VirtualGeometryPrepareFrame;

use super::super::gpu_pending_request_input::GpuPendingRequestInput;

pub(super) fn pending_requests(
    prepare: &VirtualGeometryPrepareFrame,
) -> Vec<GpuPendingRequestInput> {
    prepare
        .pending_page_requests
        .iter()
        .map(|request| GpuPendingRequestInput {
            page_id: request.page_id,
            size_bytes: request.size_bytes.min(u64::from(u32::MAX)) as u32,
            frontier_rank: request.frontier_rank,
            assigned_slot: request.assigned_slot.unwrap_or(u32::MAX),
            recycled_page_id: request.recycled_page_id.unwrap_or(u32::MAX),
        })
        .collect()
}
