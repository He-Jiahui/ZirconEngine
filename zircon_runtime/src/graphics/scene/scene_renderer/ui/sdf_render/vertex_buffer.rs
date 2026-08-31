use super::vertices::ScreenSpaceUiSdfVertex;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

const MIN_SDF_VERTEX_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfVertexBufferWriteReport {
    pub(super) capacity_byte_len: usize,
    pub(super) create_count: usize,
    pub(super) write_byte_len: usize,
}

pub(super) fn write_sdf_vertex_buffer(
    device: &wgpu::Device,
    buffer: &mut Option<wgpu::Buffer>,
    capacity_bytes: &mut u64,
    payload_hash: &mut Option<[u8; 32]>,
    vertices: &[ScreenSpaceUiSdfVertex],
    uploads: &mut WgpuBufferUploadBatch,
    force_full_upload: bool,
) -> SdfVertexBufferWriteReport {
    if vertices.is_empty() {
        *payload_hash = None;
        return SdfVertexBufferWriteReport {
            capacity_byte_len: capacity_byte_len_usize(*capacity_bytes),
            ..Default::default()
        };
    }

    let required_byte_len = std::mem::size_of_val(vertices);
    let requires_reallocation = buffer.is_none() || *capacity_bytes < required_byte_len as u64;
    if requires_reallocation {
        *capacity_bytes = sdf_vertex_buffer_capacity(required_byte_len);
        *buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-screen-space-ui-sdf-vertices"),
            size: *capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
    let vertex_bytes = bytemuck::cast_slice(vertices);
    let next_payload_hash = *blake3::hash(vertex_bytes).as_bytes();
    let payload_changed = *payload_hash != Some(next_payload_hash);
    let write_required = requires_reallocation || force_full_upload || payload_changed;
    if write_required {
        if let Some(buffer) = buffer.as_ref() {
            uploads.push(WgpuBufferUpload::from_bytes(
                buffer.clone(),
                0,
                vertex_bytes,
            ));
        }
        *payload_hash = Some(next_payload_hash);
    }
    SdfVertexBufferWriteReport {
        capacity_byte_len: capacity_byte_len_usize(*capacity_bytes),
        create_count: usize::from(requires_reallocation),
        write_byte_len: write_required.then_some(required_byte_len).unwrap_or(0),
    }
}

fn sdf_vertex_buffer_capacity(required_byte_len: usize) -> u64 {
    if required_byte_len == 0 {
        return 0;
    }
    let required = (required_byte_len as u64).max(MIN_SDF_VERTEX_BUFFER_CAPACITY_BYTES);
    required.checked_next_power_of_two().unwrap_or(required)
}

fn capacity_byte_len_usize(capacity_bytes: u64) -> usize {
    capacity_bytes.min(usize::MAX as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::sdf_vertex_buffer_capacity;

    #[test]
    fn sdf_vertex_buffer_capacity_grows_by_power_of_two_and_never_shrinks_requirement() {
        assert_eq!(sdf_vertex_buffer_capacity(0), 0);
        assert_eq!(sdf_vertex_buffer_capacity(1), 4 * 1024);
        assert_eq!(sdf_vertex_buffer_capacity(4 * 1024), 4 * 1024);
        assert_eq!(sdf_vertex_buffer_capacity(4 * 1024 + 1), 8 * 1024);
    }
}
