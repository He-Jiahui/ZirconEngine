use super::buffer_size_for_words;

pub(in crate::virtual_geometry::renderer::gpu_resources) fn create_readback_buffer(
    device: &wgpu::Device,
    label: &'static str,
    word_count: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: buffer_size_for_words(word_count.max(1)),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}
