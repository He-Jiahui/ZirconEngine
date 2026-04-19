use super::super::super::buffer_helpers::{
    create_pod_storage_buffer, create_readback_buffer, create_u32_storage_buffer,
};
use super::virtual_geometry_prepare_execution_buffers::VirtualGeometryPrepareExecutionBuffers;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

pub(super) fn create_buffers(
    device: &wgpu::Device,
    inputs: &VirtualGeometryPrepareExecutionInputs,
) -> VirtualGeometryPrepareExecutionBuffers {
    let page_table_buffer = create_u32_storage_buffer(
        device,
        "zircon-vg-page-table-buffer",
        &inputs.page_table_words,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let page_table_readback = create_readback_buffer(
        device,
        "zircon-vg-page-table-readback",
        inputs.page_table_word_count,
    );
    let request_buffer = create_pod_storage_buffer(
        device,
        "zircon-vg-request-buffer",
        &inputs.pending_requests,
        wgpu::BufferUsages::STORAGE,
    );
    let available_slot_buffer = create_u32_storage_buffer(
        device,
        "zircon-vg-available-slot-buffer",
        &inputs.available_slots,
        wgpu::BufferUsages::STORAGE,
    );
    let evictable_slot_buffer = create_u32_storage_buffer(
        device,
        "zircon-vg-evictable-slot-buffer",
        &inputs.evictable_slots,
        wgpu::BufferUsages::STORAGE,
    );
    let completed_buffer = create_u32_storage_buffer(
        device,
        "zircon-vg-completed-buffer",
        &vec![0_u32; inputs.completed_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let completed_readback = create_readback_buffer(
        device,
        "zircon-vg-completed-readback",
        inputs.completed_word_count.max(1),
    );

    VirtualGeometryPrepareExecutionBuffers {
        page_table_buffer,
        page_table_readback,
        request_buffer,
        available_slot_buffer,
        evictable_slot_buffer,
        completed_buffer,
        completed_readback,
    }
}
