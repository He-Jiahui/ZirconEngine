use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const U32_SIZE: u64 = std::mem::size_of::<u32>() as u64;

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

pub(super) fn create_readback_buffer(
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

pub(super) fn buffer_size_for_words(word_count: usize) -> u64 {
    (word_count.max(1) as u64) * U32_SIZE
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
