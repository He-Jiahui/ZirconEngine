use super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;

pub(super) fn create_bind_group(
    resources: &HybridGiGpuResources,
    device: &wgpu::Device,
    buffers: &HybridGiPrepareExecutionBuffers,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-completion-bind-group"),
        layout: &resources.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: resources.params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.resident_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.pending_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.trace_region_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.completed_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: buffers.completed_trace_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: buffers.irradiance_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: buffers.trace_lighting_buffer.as_entire_binding(),
            },
        ],
    })
}
