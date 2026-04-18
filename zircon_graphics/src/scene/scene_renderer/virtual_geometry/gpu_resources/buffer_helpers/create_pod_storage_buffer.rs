use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) fn create_pod_storage_buffer<
    T: Pod + Zeroable,
>(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    if contents.is_empty() {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&T::zeroed()),
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage: usage | wgpu::BufferUsages::COPY_DST,
    })
}
