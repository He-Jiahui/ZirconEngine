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
