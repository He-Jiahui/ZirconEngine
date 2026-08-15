#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct IndirectBufferUploadStats {
    pub(super) byte_count: u64,
    pub(super) range_count: u32,
}

pub(super) fn write_changed_pod_ranges<T>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    shadow: &mut Vec<T>,
    current: &[T],
    force_full_upload: bool,
) -> IndirectBufferUploadStats
where
    T: bytemuck::Pod + Copy + PartialEq,
{
    if force_full_upload {
        let stats = write_range(queue, buffer, 0, current);
        shadow.clear();
        shadow.extend_from_slice(current);
        return stats;
    }

    let shared_len = shadow.len().min(current.len());
    let mut stats = IndirectBufferUploadStats::default();
    let mut cursor = 0usize;
    while cursor < shared_len {
        while cursor < shared_len && shadow[cursor] == current[cursor] {
            cursor += 1;
        }
        let dirty_start = cursor;
        while cursor < shared_len && shadow[cursor] != current[cursor] {
            cursor += 1;
        }
        if dirty_start != cursor {
            accumulate_upload(
                &mut stats,
                write_range(queue, buffer, dirty_start, &current[dirty_start..cursor]),
            );
            shadow[dirty_start..cursor].copy_from_slice(&current[dirty_start..cursor]);
        }
    }

    if current.len() > shared_len {
        accumulate_upload(
            &mut stats,
            write_range(queue, buffer, shared_len, &current[shared_len..]),
        );
        shadow.extend_from_slice(&current[shared_len..]);
    } else {
        shadow.truncate(current.len());
    }
    stats
}

fn write_range<T: bytemuck::Pod>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    element_offset: usize,
    values: &[T],
) -> IndirectBufferUploadStats {
    if values.is_empty() {
        return IndirectBufferUploadStats::default();
    }
    let element_size = std::mem::size_of::<T>() as wgpu::BufferAddress;
    let offset = element_offset as wgpu::BufferAddress * element_size;
    let bytes = bytemuck::cast_slice(values);
    queue.write_buffer(buffer, offset, bytes);
    IndirectBufferUploadStats {
        byte_count: bytes.len() as u64,
        range_count: 1,
    }
}

fn accumulate_upload(total: &mut IndirectBufferUploadStats, next: IndirectBufferUploadStats) {
    total.byte_count = total.byte_count.saturating_add(next.byte_count);
    total.range_count = total.range_count.saturating_add(next.range_count);
}
