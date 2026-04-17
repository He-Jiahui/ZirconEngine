use crate::types::VirtualGeometryPrepareFrame;

use super::super::super::virtual_geometry_gpu_resources::VirtualGeometryGpuResources;
use super::super::super::virtual_geometry_uploader_params::VirtualGeometryUploaderParams;
use super::super::reclaimable_bytes::reclaimable_bytes;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

const VIRTUAL_GEOMETRY_STREAMING_PAGE_GRANULARITY_BYTES: u64 = 4_096;

pub(super) fn queue_params(
    resources: &VirtualGeometryGpuResources,
    queue: &wgpu::Queue,
    prepare: &VirtualGeometryPrepareFrame,
    inputs: &VirtualGeometryPrepareExecutionInputs,
    page_budget: Option<u32>,
) {
    let page_budget = page_budget.unwrap_or_default();
    let params = VirtualGeometryUploaderParams {
        pending_count: inputs.pending_requests.len() as u32,
        available_slot_count: inputs.available_slots.len() as u32,
        evictable_count: prepare.evictable_pages.len() as u32,
        page_budget,
        streaming_budget_bytes: page_budget.saturating_mul(
            VIRTUAL_GEOMETRY_STREAMING_PAGE_GRANULARITY_BYTES.min(u64::from(u32::MAX)) as u32,
        ),
        reclaimable_bytes: reclaimable_bytes(prepare),
        resident_count: inputs.resident_entries.len() as u32,
        _padding1: 0,
    };
    queue.write_buffer(&resources.params_buffer, 0, bytemuck::bytes_of(&params));
}
