use zr_rhi::{BufferHandle, BufferUsage, RhiError, TextureCopyRegion, TextureHandle, TextureUsage};

use crate::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use crate::texture_copy::{
    texture_copy_layout, texture_to_texture_copy_layouts, validate_texture_copy_destination_aspect,
    validate_texture_copy_source_aspect,
};

use super::super::registry::WgpuResourceRegistry;
use super::super::translate::wgpu_texture_copy_aspect;

const BUFFER_COPY_ALIGNMENT: u64 = wgpu::COPY_BUFFER_ALIGNMENT as u64;
const TEXTURE_ROW_ALIGNMENT: u64 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;

/// Encodes an admitted neutral buffer-to-texture copy. The shared destination
/// validator keeps portable depth formats fail-closed because WGPU forbids
/// buffer writes to their depth aspect.
pub(crate) fn encode_buffer_to_texture_copy(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    source: BufferHandle,
    destination: TextureHandle,
    source_offset: u64,
    bytes_per_row: u64,
    region: TextureCopyRegion,
) -> Result<(), RhiError> {
    let source_desc = registry.buffer_desc(source)?;
    let destination_desc = registry.texture_desc(destination)?;
    ensure_buffer_usage(source.diagnostic_id(), &source_desc, BufferUsage::COPY_SRC)?;
    ensure_texture_usage(
        destination.diagnostic_id(),
        &destination_desc,
        TextureUsage::COPY_DST,
    )?;
    validate_texture_copy_destination_aspect(destination, &destination_desc, region)?;
    let copy_layout = texture_copy_layout(&destination_desc, region).ok_or_else(|| {
        buffer_to_texture_range_error(source, destination, source_offset, bytes_per_row, region)
    })?;
    let native_bytes_per_row = validate_wgpu_buffer_texture_layout(
        source_offset,
        bytes_per_row,
        copy_layout.copy_row_bytes,
        region,
        source_desc.size_bytes,
    )
    .ok_or_else(|| {
        buffer_to_texture_range_error(source, destination, source_offset, bytes_per_row, region)
    })?;

    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: registry.buffer(source)?,
            layout: wgpu::TexelCopyBufferLayout {
                offset: source_offset,
                bytes_per_row: Some(native_bytes_per_row),
                rows_per_image: Some(region.height),
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture: registry.texture(destination)?,
            mip_level: region.mip_level,
            origin: wgpu::Origin3d {
                x: region.origin_x,
                y: region.origin_y,
                z: region.origin_z,
            },
            aspect: wgpu_texture_copy_aspect(region.aspect),
        },
        wgpu::Extent3d {
            width: region.width,
            height: region.height,
            depth_or_array_layers: region.depth_or_array_layers,
        },
    );
    Ok(())
}

/// Encodes an admitted neutral texture-to-buffer copy. The destination is an
/// ordinary COPY_DST buffer; mapping and public diagnostic delivery remain a
/// separate submission-qualified service.
pub(crate) fn encode_texture_to_buffer_copy(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    source: TextureHandle,
    destination: BufferHandle,
    destination_offset: u64,
    bytes_per_row: u64,
    region: TextureCopyRegion,
) -> Result<(), RhiError> {
    let source_desc = registry.texture_desc(source)?;
    let destination_desc = registry.buffer_desc(destination)?;
    ensure_texture_usage(source.diagnostic_id(), &source_desc, TextureUsage::COPY_SRC)?;
    ensure_buffer_usage(
        destination.diagnostic_id(),
        &destination_desc,
        BufferUsage::COPY_DST,
    )?;
    validate_texture_copy_source_aspect(source, &source_desc, region)?;
    let copy_layout = texture_copy_layout(&source_desc, region).ok_or_else(|| {
        texture_to_buffer_range_error(
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        )
    })?;
    let native_bytes_per_row = validate_wgpu_buffer_texture_layout(
        destination_offset,
        bytes_per_row,
        copy_layout.copy_row_bytes,
        region,
        destination_desc.size_bytes,
    )
    .ok_or_else(|| {
        texture_to_buffer_range_error(
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        )
    })?;

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: registry.texture(source)?,
            mip_level: region.mip_level,
            origin: wgpu::Origin3d {
                x: region.origin_x,
                y: region.origin_y,
                z: region.origin_z,
            },
            aspect: wgpu_texture_copy_aspect(region.aspect),
        },
        wgpu::TexelCopyBufferInfo {
            buffer: registry.buffer(destination)?,
            layout: wgpu::TexelCopyBufferLayout {
                offset: destination_offset,
                bytes_per_row: Some(native_bytes_per_row),
                rows_per_image: Some(region.height),
            },
        },
        wgpu::Extent3d {
            width: region.width,
            height: region.height,
            depth_or_array_layers: region.depth_or_array_layers,
        },
    );
    Ok(())
}

/// Encodes a conservative color texture subresource copy used by physical mip
/// residency transitions. The shared contract rejects reinterpretation,
/// depth/stencil, multisample, aliasing, and mismatched extents before WGPU
/// sees the command.
pub(crate) fn encode_texture_to_texture_copy(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    source: TextureHandle,
    destination: TextureHandle,
    source_region: TextureCopyRegion,
    destination_region: TextureCopyRegion,
) -> Result<(), RhiError> {
    let source_desc = registry.texture_desc(source)?;
    let destination_desc = registry.texture_desc(destination)?;
    ensure_texture_usage(source.diagnostic_id(), &source_desc, TextureUsage::COPY_SRC)?;
    ensure_texture_usage(
        destination.diagnostic_id(),
        &destination_desc,
        TextureUsage::COPY_DST,
    )?;
    texture_to_texture_copy_layouts(
        source,
        &source_desc,
        destination,
        &destination_desc,
        source_region,
        destination_region,
    )?;

    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: registry.texture(source)?,
            mip_level: source_region.mip_level,
            origin: wgpu::Origin3d {
                x: source_region.origin_x,
                y: source_region.origin_y,
                z: source_region.origin_z,
            },
            aspect: wgpu_texture_copy_aspect(source_region.aspect),
        },
        wgpu::TexelCopyTextureInfo {
            texture: registry.texture(destination)?,
            mip_level: destination_region.mip_level,
            origin: wgpu::Origin3d {
                x: destination_region.origin_x,
                y: destination_region.origin_y,
                z: destination_region.origin_z,
            },
            aspect: wgpu_texture_copy_aspect(destination_region.aspect),
        },
        wgpu::Extent3d {
            width: source_region.width,
            height: source_region.height,
            depth_or_array_layers: source_region.depth_or_array_layers,
        },
    );
    Ok(())
}

fn validate_wgpu_buffer_texture_layout(
    offset: u64,
    bytes_per_row: u64,
    copy_row_bytes: u64,
    region: TextureCopyRegion,
    buffer_size: u64,
) -> Option<u32> {
    let native_bytes_per_row = u32::try_from(bytes_per_row).ok()?;
    let texel_block_bytes = copy_row_bytes.checked_div(u64::from(region.width))?;
    let required_bytes = copy_byte_len(
        region.depth_or_array_layers,
        region.height,
        bytes_per_row,
        copy_row_bytes,
    )?;
    (offset % BUFFER_COPY_ALIGNMENT == 0
        && bytes_per_row >= copy_row_bytes
        && bytes_per_row % texel_block_bytes == 0
        && ((region.height <= 1 && region.depth_or_array_layers <= 1)
            || bytes_per_row % TEXTURE_ROW_ALIGNMENT == 0)
        && offset.checked_add(required_bytes)? <= buffer_size)
        .then_some(native_bytes_per_row)
}

fn copy_byte_len(
    depth_or_array_layers: u32,
    rows_per_image: u32,
    bytes_per_row: u64,
    copy_row_bytes: u64,
) -> Option<u64> {
    let preceding_slices = u64::from(depth_or_array_layers)
        .checked_sub(1)?
        .checked_mul(u64::from(rows_per_image))?
        .checked_mul(bytes_per_row)?;
    let final_slice_rows = u64::from(rows_per_image)
        .checked_sub(1)?
        .checked_mul(bytes_per_row)?
        .checked_add(copy_row_bytes)?;
    preceding_slices.checked_add(final_slice_rows)
}

fn buffer_to_texture_range_error(
    source: BufferHandle,
    destination: TextureHandle,
    source_offset: u64,
    bytes_per_row: u64,
    region: TextureCopyRegion,
) -> RhiError {
    RhiError::BufferToTextureCopyOutOfRange {
        source_buffer: source.diagnostic_id(),
        destination_texture: destination.diagnostic_id(),
        source_offset,
        bytes_per_row,
        mip_level: region.mip_level,
        origin_x: region.origin_x,
        origin_y: region.origin_y,
        origin_z: region.origin_z,
        width: region.width,
        height: region.height,
        depth_or_array_layers: region.depth_or_array_layers,
    }
}

fn texture_to_buffer_range_error(
    source: TextureHandle,
    destination: BufferHandle,
    destination_offset: u64,
    bytes_per_row: u64,
    region: TextureCopyRegion,
) -> RhiError {
    RhiError::TextureToBufferCopyOutOfRange {
        source_texture: source.diagnostic_id(),
        destination_buffer: destination.diagnostic_id(),
        destination_offset,
        bytes_per_row,
        mip_level: region.mip_level,
        origin_x: region.origin_x,
        origin_y: region.origin_y,
        origin_z: region.origin_z,
        width: region.width,
        height: region.height,
        depth_or_array_layers: region.depth_or_array_layers,
    }
}
