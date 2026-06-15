use wgpu::util::DeviceExt;

use super::super::super::super::params::exposure_params::default_exposure_buffer_words;

pub(super) fn default_exposure_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-default-exposure-buffer"),
        contents: bytemuck::cast_slice(&default_exposure_buffer_words()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}
