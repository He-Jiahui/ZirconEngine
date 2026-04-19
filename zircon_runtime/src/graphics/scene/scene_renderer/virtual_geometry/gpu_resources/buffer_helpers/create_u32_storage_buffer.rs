use wgpu::util::DeviceExt;

pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) fn create_u32_storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[u32],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let contents = if contents.is_empty() { &[0] } else { contents };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage: usage | wgpu::BufferUsages::COPY_DST,
    })
}
