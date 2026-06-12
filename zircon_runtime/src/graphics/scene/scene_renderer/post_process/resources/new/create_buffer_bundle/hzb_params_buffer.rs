use super::super::super::super::params::hzb_params::HzbParams;

pub(super) fn hzb_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hzb-params"),
        size: std::mem::size_of::<HzbParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
