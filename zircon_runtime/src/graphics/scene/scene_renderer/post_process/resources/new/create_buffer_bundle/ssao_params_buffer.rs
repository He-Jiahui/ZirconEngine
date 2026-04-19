use super::super::super::super::ssao_params::SsaoParams;

pub(super) fn ssao_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ssao-params"),
        size: std::mem::size_of::<SsaoParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
