use zr_rhi::{
    RhiError, TextureCopyAspect, TextureCopyRegion, TextureDesc, TextureDimension, TextureHandle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextureCopyLayout {
    pub offset: u64,
    pub row_stride: u64,
    pub slice_stride: u64,
    pub copy_row_bytes: u64,
    pub last_copy_end: u64,
}

/// Validates a CPU-upload row layout independently from WGPU command-encoder
/// alignment rules. Queue writes accept tightly packed multiline rows, while
/// encoded buffer copies require 256-byte alignment.
pub(crate) fn texture_upload_layout(
    desc: &TextureDesc,
    region: TextureCopyRegion,
    bytes_per_row: u64,
    source_bytes: u64,
) -> Option<TextureCopyLayout> {
    let layout = texture_copy_layout(desc, region)?;
    let texel_block_bytes = layout.copy_row_bytes.checked_div(u64::from(region.width))?;
    let required_bytes = copy_byte_len(
        region.depth_or_array_layers,
        region.height,
        bytes_per_row,
        layout.copy_row_bytes,
    )?;
    (bytes_per_row >= layout.copy_row_bytes
        && bytes_per_row % texel_block_bytes == 0
        && source_bytes >= required_bytes)
        .then_some(layout)
}

pub(crate) fn texture_upload_byte_len(
    region: TextureCopyRegion,
    bytes_per_row: u64,
    copy_row_bytes: u64,
) -> Option<u64> {
    copy_byte_len(
        region.depth_or_array_layers,
        region.height,
        bytes_per_row,
        copy_row_bytes,
    )
}

pub(crate) fn texture_write_out_of_range(
    texture: TextureHandle,
    source_bytes: u64,
    bytes_per_row: u64,
    region: TextureCopyRegion,
) -> RhiError {
    RhiError::TextureWriteOutOfRange {
        texture: texture.diagnostic_id(),
        source_bytes,
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

/// Validates the requested aspect against the source side of WGPU's linear
/// texture-copy contract. `Depth32Float` is copyable only from its explicit
/// depth aspect; portable `Depth24Plus` formats remain fail-closed.
pub(crate) fn validate_texture_copy_source_aspect(
    texture: TextureHandle,
    desc: &TextureDesc,
    region: TextureCopyRegion,
) -> Result<(), RhiError> {
    if texture_copy_bytes_per_texel(desc, region.aspect).is_some() {
        return Ok(());
    }
    Err(RhiError::InvalidCopy {
        reason: format!(
            "texture copy source {:?} format {:?} does not support aspect {:?}",
            texture.diagnostic_id(),
            desc.format,
            region.aspect
        ),
    })
}

/// Validates the requested aspect against the destination side of WGPU's
/// linear texture-copy contract. WebGPU forbids writes to portable depth
/// formats, so only a color `All` aspect is currently admitted.
pub(crate) fn validate_texture_copy_destination_aspect(
    texture: TextureHandle,
    desc: &TextureDesc,
    region: TextureCopyRegion,
) -> Result<(), RhiError> {
    if !desc.format.is_depth() && region.aspect == TextureCopyAspect::All {
        return Ok(());
    }
    Err(RhiError::InvalidCopy {
        reason: format!(
            "texture copy destination {:?} format {:?} does not support aspect {:?}",
            texture.diagnostic_id(),
            desc.format,
            region.aspect
        ),
    })
}

/// Validates the MVP texture-to-texture path shared by deterministic and WGPU
/// executors. It intentionally admits only exact-format, single-sampled color
/// copies between distinct physical textures; view reinterpretation and
/// depth/stencil transfers need their own resource/aspect contracts.
pub(crate) fn texture_to_texture_copy_layouts(
    source: TextureHandle,
    source_desc: &TextureDesc,
    destination: TextureHandle,
    destination_desc: &TextureDesc,
    source_region: TextureCopyRegion,
    destination_region: TextureCopyRegion,
) -> Result<(TextureCopyLayout, TextureCopyLayout), RhiError> {
    if source == destination {
        return Err(RhiError::InvalidCopy {
            reason: "texture copies must use distinct source and destination textures".to_string(),
        });
    }
    if source_desc.dimension != destination_desc.dimension
        || source_desc.format != destination_desc.format
        || source_desc.sample_count != 1
        || destination_desc.sample_count != 1
    {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "texture copy {:?} -> {:?} requires matching dimension, format, and single-sample descriptors",
                source.diagnostic_id(),
                destination.diagnostic_id(),
            ),
        });
    }
    if source_desc.format.is_depth()
        || source_region.aspect != TextureCopyAspect::All
        || destination_region.aspect != TextureCopyAspect::All
    {
        return Err(RhiError::InvalidCopy {
            reason:
                "texture-to-texture copies currently support only color All-aspect subresources"
                    .to_string(),
        });
    }
    if source_region.width != destination_region.width
        || source_region.height != destination_region.height
        || source_region.depth_or_array_layers != destination_region.depth_or_array_layers
    {
        return Err(RhiError::InvalidCopy {
            reason: "texture source and destination copy extents must match".to_string(),
        });
    }

    let source_layout =
        texture_copy_layout(source_desc, source_region).ok_or_else(|| RhiError::InvalidCopy {
            reason: format!(
                "texture copy source {:?} region is outside its descriptor",
                source.diagnostic_id(),
            ),
        })?;
    let destination_layout =
        texture_copy_layout(destination_desc, destination_region).ok_or_else(|| {
            RhiError::InvalidCopy {
                reason: format!(
                    "texture copy destination {:?} region is outside its descriptor",
                    destination.diagnostic_id(),
                ),
            }
        })?;
    Ok((source_layout, destination_layout))
}

pub(crate) fn texture_copy_layout(
    desc: &TextureDesc,
    region: TextureCopyRegion,
) -> Option<TextureCopyLayout> {
    if desc.is_sparse_reserved()
        || desc.sample_count != 1
        || region.width == 0
        || region.height == 0
        || region.depth_or_array_layers == 0
        || region.mip_level >= desc.mip_levels
    {
        return None;
    }

    let mip_width = mip_extent(desc.width, region.mip_level);
    let mip_height = mip_extent(desc.height, region.mip_level);
    let mip_depth = texture_mip_depth(desc, region.mip_level);
    let bytes_per_pixel = u64::from(texture_copy_bytes_per_texel(desc, region.aspect)?);
    let x_end = region.origin_x.checked_add(region.width)?;
    let y_end = region.origin_y.checked_add(region.height)?;
    let z_end = region.origin_z.checked_add(region.depth_or_array_layers)?;
    if x_end > mip_width || y_end > mip_height || z_end > mip_depth {
        return None;
    }
    if desc.format.is_depth()
        && (region.origin_x != 0
            || region.origin_y != 0
            || region.width != mip_width
            || region.height != mip_height
            || region.depth_or_array_layers != 1)
    {
        return None;
    }

    let row_stride = u64::from(mip_width).checked_mul(bytes_per_pixel)?;
    let slice_stride = row_stride.checked_mul(u64::from(mip_height))?;
    let level_offset = texture_mip_level_offset(desc, region.mip_level)?;
    let slice_offset = u64::from(region.origin_z).checked_mul(slice_stride)?;
    let row_offset = u64::from(region.origin_y).checked_mul(row_stride)?;
    let column_offset = u64::from(region.origin_x).checked_mul(bytes_per_pixel)?;
    let offset = level_offset
        .checked_add(slice_offset)?
        .checked_add(row_offset)?
        .checked_add(column_offset)?;
    let copy_row_bytes = u64::from(region.width).checked_mul(bytes_per_pixel)?;
    let last_slice_offset =
        u64::from(region.depth_or_array_layers.saturating_sub(1)).checked_mul(slice_stride)?;
    let last_row_offset = u64::from(region.height.saturating_sub(1)).checked_mul(row_stride)?;
    let last_copy_end = offset
        .checked_add(last_slice_offset)?
        .checked_add(last_row_offset)?
        .checked_add(copy_row_bytes)?;
    Some(TextureCopyLayout {
        offset,
        row_stride,
        slice_stride,
        copy_row_bytes,
        last_copy_end,
    })
}

fn texture_copy_bytes_per_texel(desc: &TextureDesc, aspect: TextureCopyAspect) -> Option<u32> {
    match (desc.format.is_depth(), desc.format, aspect) {
        (false, _, TextureCopyAspect::All) => Some(desc.format.bytes_per_pixel()),
        (true, zr_rhi::TextureFormat::Depth32Float, TextureCopyAspect::DepthOnly) => Some(4),
        _ => None,
    }
}

fn texture_mip_level_offset(desc: &TextureDesc, mip_level: u32) -> Option<u64> {
    let mut offset = 0_u64;
    let bytes_per_pixel = u64::from(desc.format.bytes_per_pixel());
    for level in 0..mip_level {
        let width = u64::from(mip_extent(desc.width, level));
        let height = u64::from(mip_extent(desc.height, level));
        let depth = u64::from(texture_mip_depth(desc, level));
        let level_size = width
            .checked_mul(height)?
            .checked_mul(depth)?
            .checked_mul(u64::from(desc.sample_count))?
            .checked_mul(bytes_per_pixel)?;
        offset = offset.checked_add(level_size)?;
    }
    Some(offset)
}

fn texture_mip_depth(desc: &TextureDesc, mip_level: u32) -> u32 {
    match desc.dimension {
        TextureDimension::D3 => mip_extent(desc.depth, mip_level),
        TextureDimension::D1
        | TextureDimension::D2
        | TextureDimension::D2Array
        | TextureDimension::Cube => desc.depth,
    }
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 {
        1
    } else {
        shifted
    }
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
