use super::super::super::super::constants::MAX_REFLECTION_PROBES;
use super::super::super::super::reflection_probe_gpu::GpuReflectionProbe;

pub(super) fn reflection_probe_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-reflection-probe-buffer"),
        size: (std::mem::size_of::<GpuReflectionProbe>() * MAX_REFLECTION_PROBES) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
