use crate::rhi::{TextureCopyRegion, TextureDesc, TextureDimension};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TextureCopyLayout {
    pub offset: u64,
    pub row_stride: u64,
    pub copy_row_bytes: u64,
    pub last_row_end: u64,
}

pub(super) fn texture_copy_layout(
    desc: &TextureDesc,
    region: TextureCopyRegion,
) -> Option<TextureCopyLayout> {
    if desc.is_sparse_reserved()
        || desc.sample_count != 1
        || region.width == 0
        || region.height == 0
        || region.mip_level >= desc.mip_levels
    {
        return None;
    }

    let mip_width = mip_extent(desc.width, region.mip_level);
    let mip_height = mip_extent(desc.height, region.mip_level);
    let mip_depth = texture_mip_depth(desc, region.mip_level);
    let bytes_per_pixel = u64::from(desc.format.bytes_per_pixel());
    let x_end = region.origin_x.checked_add(region.width)?;
    let y_end = region.origin_y.checked_add(region.height)?;
    let z_end = region.origin_z.checked_add(1)?;
    if x_end > mip_width || y_end > mip_height || z_end > mip_depth {
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
    let last_row_offset = u64::from(region.height.saturating_sub(1)).checked_mul(row_stride)?;
    let last_row_end = offset
        .checked_add(last_row_offset)?
        .checked_add(copy_row_bytes)?;
    Some(TextureCopyLayout {
        offset,
        row_stride,
        copy_row_bytes,
        last_row_end,
    })
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
