use wgpu::util::DeviceExt;

use super::super::super::super::params::exposure_params::default_exposure_buffer_words;

/// 在自动曝光尚未写入有效历史时提供稳定的场景曝光绑定。
/// 初值来自框架曝光契约，后处理或 LUT 烘焙可以使用它而无需空绑定分支。
pub(super) fn default_exposure_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-default-exposure-buffer"),
        contents: bytemuck::cast_slice(&default_exposure_buffer_words()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}
