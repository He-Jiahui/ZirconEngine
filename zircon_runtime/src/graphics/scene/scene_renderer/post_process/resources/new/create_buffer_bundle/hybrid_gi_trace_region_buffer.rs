use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;

pub(super) fn hybrid_gi_trace_region_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hybrid-gi-trace-region-buffer"),
        size: (std::mem::size_of::<GpuHybridGiTraceRegion>() * MAX_HYBRID_GI_TRACE_REGIONS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
