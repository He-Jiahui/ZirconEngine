use super::super::super::virtual_geometry_gpu_resources::VirtualGeometryGpuResources;
use super::virtual_geometry_prepare_execution_buffers::VirtualGeometryPrepareExecutionBuffers;

pub(super) fn create_bind_group(
    resources: &VirtualGeometryGpuResources,
    device: &wgpu::Device,
    buffers: &VirtualGeometryPrepareExecutionBuffers,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-vg-uploader-bind-group"),
        layout: &resources.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: resources.params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.request_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.available_slot_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.evictable_slot_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.completed_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: buffers.page_table_buffer.as_entire_binding(),
            },
        ],
    })
}
