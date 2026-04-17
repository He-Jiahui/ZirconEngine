use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;

pub(super) fn hybrid_gi_probe_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hybrid-gi-probe-buffer"),
        size: (std::mem::size_of::<GpuHybridGiProbe>() * MAX_HYBRID_GI_PROBES) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
