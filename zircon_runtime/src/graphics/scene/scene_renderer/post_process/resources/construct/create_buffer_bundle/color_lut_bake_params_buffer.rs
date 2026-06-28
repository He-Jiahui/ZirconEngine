use super::super::super::super::params::color_lut_bake_params::ColorLutBakeParams;

pub(super) fn color_lut_bake_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-color-lut-bake-params"),
        size: std::mem::size_of::<ColorLutBakeParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
