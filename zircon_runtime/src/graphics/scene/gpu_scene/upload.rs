use bytemuck::Pod;

use super::update_queue::GpuSceneUploadRange;

pub(super) fn write_full_pod_buffer<T: Pod>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    shadow: &[T],
    active_len: usize,
) -> u64 {
    if active_len == 0 {
        return 0;
    }
    assert!(
        active_len <= shadow.len(),
        "gpu scene full upload length exceeds CPU shadow length"
    );
    let active = &shadow[..active_len];
    let bytes = bytemuck::cast_slice(active);
    if bytes.is_empty() {
        return 0;
    }
    queue.write_buffer(buffer, 0, bytes);
    bytes.len() as u64
}

pub(super) fn write_changed_pod_buffer<T: Pod + PartialEq>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    previous: &[T],
    current: &[T],
) -> u64 {
    let mut uploaded_bytes = 0;
    let mut index = 0;
    while index < current.len() {
        if previous.get(index) == Some(&current[index]) {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < current.len() && previous.get(index) != Some(&current[index]) {
            index += 1;
        }

        let bytes = bytemuck::cast_slice(&current[start..index]);
        if bytes.is_empty() {
            continue;
        }
        let byte_offset = start
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|offset| u64::try_from(offset).ok())
            .expect("gpu scene dirty upload offset overflowed u64");
        queue.write_buffer(buffer, byte_offset, bytes);
        uploaded_bytes += bytes.len() as u64;
    }
    uploaded_bytes
}

pub(super) fn write_upload_ranges<T: Pod>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    shadow: &[T],
    ranges: &[GpuSceneUploadRange],
) -> u64 {
    let mut uploaded_bytes = 0;
    for range in ranges {
        let start = range.start as usize;
        let end = start
            .checked_add(range.len as usize)
            .expect("gpu scene upload range overflowed usize");
        let bytes = bytemuck::cast_slice(&shadow[start..end]);
        queue.write_buffer(buffer, range.byte_offset, bytes);
        uploaded_bytes += bytes.len() as u64;
    }
    uploaded_bytes
}
