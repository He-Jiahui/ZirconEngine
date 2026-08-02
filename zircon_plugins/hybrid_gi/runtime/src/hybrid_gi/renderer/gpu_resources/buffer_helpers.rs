use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub(super) fn create_u32_storage_buffer(
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

pub(super) fn create_pod_storage_buffer<T: Pod + Zeroable>(
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
