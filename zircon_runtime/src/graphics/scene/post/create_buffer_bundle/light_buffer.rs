use super::super::super::super::clustered_directional_light::ClusteredDirectionalLight;
use super::super::super::super::constants::MAX_DIRECTIONAL_LIGHTS;

pub(super) fn light_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-directional-light-buffer"),
        size: (std::mem::size_of::<ClusteredDirectionalLight>() * MAX_DIRECTIONAL_LIGHTS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
