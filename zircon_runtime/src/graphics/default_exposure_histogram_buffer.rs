use wgpu::util::DeviceExt;

use crate::core::framework::render::EXPOSURE_HISTOGRAM_BIN_COUNT;

/// 为不运行直方图阶段的帧提供零计数存储缓冲，保持曝光解析绑定完整。
pub(super) fn default_exposure_histogram_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let histogram = [0_u32; EXPOSURE_HISTOGRAM_BIN_COUNT as usize];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-default-exposure-histogram-buffer"),
        contents: bytemuck::cast_slice(&histogram),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}
