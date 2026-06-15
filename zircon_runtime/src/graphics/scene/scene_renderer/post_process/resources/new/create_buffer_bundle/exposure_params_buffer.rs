use super::super::super::super::params::exposure_params::ExposureParams;

pub(super) fn exposure_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-exposure-params"),
        size: std::mem::size_of::<ExposureParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
