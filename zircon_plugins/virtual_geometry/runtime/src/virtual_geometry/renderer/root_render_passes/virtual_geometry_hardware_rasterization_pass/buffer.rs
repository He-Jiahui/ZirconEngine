use std::sync::Arc;

use wgpu::util::DeviceExt;

pub(super) fn create_hardware_rasterization_buffer(
    device: &wgpu::Device,
    packed_words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-hardware-rasterization-pass-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}
