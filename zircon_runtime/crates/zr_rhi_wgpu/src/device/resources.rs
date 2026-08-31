use zr_rhi::{
    BufferHandle, BufferUsage, GpuMemoryClass, RhiError, TextureCopyRegion, TextureHandle,
    TextureUsage,
};

use crate::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use crate::texture_copy::{
    texture_upload_layout, texture_write_out_of_range, validate_texture_copy_destination_aspect,
};

use super::DeterministicRhiContractDeviceState;

pub(super) fn ensure_memory_capacity(
    class: GpuMemoryClass,
    current_bytes: u64,
    requested_bytes: u64,
    limit_bytes: u64,
) -> Result<(), RhiError> {
    if requested_bytes > limit_bytes.saturating_sub(current_bytes) {
        return Err(RhiError::MemoryBudgetExceeded {
            class,
            current_bytes,
            requested_bytes,
            limit_bytes,
        });
    }
    Ok(())
}

pub(super) fn allocate_zeroed_contents(
    class: GpuMemoryClass,
    requested_bytes: u64,
) -> Result<Vec<u8>, RhiError> {
    let length =
        usize::try_from(requested_bytes).map_err(|_| RhiError::ResourceAllocationFailed {
            class,
            requested_bytes,
        })?;
    let mut contents = Vec::new();
    contents
        .try_reserve_exact(length)
        .map_err(|_| RhiError::ResourceAllocationFailed {
            class,
            requested_bytes,
        })?;
    contents.resize(length, 0);
    Ok(contents)
}

pub(super) fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(super) fn execute_buffer_upload(
    state: &mut DeterministicRhiContractDeviceState,
    handle: BufferHandle,
    offset: u64,
    data: &[u8],
) -> Result<(), RhiError> {
    let buffer = state
        .buffers
        .get_mut(&handle)
        .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))?;
    ensure_buffer_usage(handle.diagnostic_id(), &buffer.desc, BufferUsage::COPY_DST)?;
    let size = data.len() as u64;
    if offset.saturating_add(size) > buffer.desc.size_bytes {
        return Err(RhiError::WriteOutOfRange {
            buffer: handle.diagnostic_id(),
            offset,
            size,
        });
    }
    let start = offset as usize;
    let end = start + data.len();
    buffer.contents[start..end].copy_from_slice(data);
    Ok(())
}

pub(super) fn execute_texture_upload(
    state: &mut DeterministicRhiContractDeviceState,
    handle: TextureHandle,
    region: TextureCopyRegion,
    bytes_per_row: u64,
    data: &[u8],
) -> Result<(), RhiError> {
    let texture = state
        .textures
        .get_mut(&handle)
        .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))?;
    ensure_texture_usage(
        handle.diagnostic_id(),
        &texture.desc,
        TextureUsage::COPY_DST,
    )?;
    validate_texture_copy_destination_aspect(handle, &texture.desc, region)?;
    let source_bytes = data.len() as u64;
    let layout = texture_upload_layout(&texture.desc, region, bytes_per_row, source_bytes)
        .ok_or_else(|| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
    for slice in 0..u64::from(region.depth_or_array_layers) {
        for row in 0..u64::from(region.height) {
            let source_row = slice * u64::from(region.height) + row;
            let source_start = (source_row * bytes_per_row) as usize;
            let destination_start =
                (layout.offset + slice * layout.slice_stride + row * layout.row_stride) as usize;
            let row_size = layout.copy_row_bytes as usize;
            texture.contents[destination_start..destination_start + row_size]
                .copy_from_slice(&data[source_start..source_start + row_size]);
        }
    }
    Ok(())
}
