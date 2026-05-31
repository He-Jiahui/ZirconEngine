use super::{
    support::{parse_error, parse_error_value, read_u32_le, KTX2_LEVEL_INDEX_ENTRY_SIZE},
    TextureContainerInfo,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

mod ktx1;
mod ktx2;

pub(super) fn parse_ktx1(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    ktx1::parse(context)
}

pub(super) fn parse_ktx2(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    ktx2::parse(context)
}

fn texture_dimension_from_header(height: u32, depth: u32) -> RenderImageDimension {
    if depth > 0 {
        RenderImageDimension::D3
    } else if height == 0 {
        RenderImageDimension::D1
    } else {
        RenderImageDimension::D2
    }
}

fn validate_3d_height(
    context: &AssetImportContext,
    label: &str,
    height: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && height == 0 {
        return parse_error(
            context,
            format!("{label} 3d texture height must be nonzero when depth is nonzero"),
        );
    }
    Ok(())
}

fn validate_cubemap_depth(
    context: &AssetImportContext,
    label: &str,
    face_count: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && face_count == KTX_CUBEMAP_FACE_COUNT {
        return parse_error(
            context,
            format!("{label} cubemap textures must not declare 3d depth"),
        );
    }
    Ok(())
}

fn validate_cubemap_2d_square_faces(
    context: &AssetImportContext,
    label: &str,
    face_count: u32,
    width: u32,
    height: u32,
) -> Result<(), AssetImportError> {
    if face_count == KTX_CUBEMAP_FACE_COUNT && (height == 0 || width != height) {
        return parse_error(
            context,
            format!("{label} cubemap faces must be 2d and square, got {width}x{height}"),
        );
    }
    Ok(())
}

fn validate_3d_array_layers(
    context: &AssetImportContext,
    label: &str,
    declared_layer_count: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && declared_layer_count > 0 {
        return parse_error(
            context,
            format!("{label} 3d textures must not declare array layers"),
        );
    }
    Ok(())
}

fn texture_array_layers(dimension: RenderImageDimension, array_layers: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        array_layers.max(1)
    }
}

fn validate_mip_count_fits_extent(
    context: &AssetImportContext,
    label: &str,
    width: u32,
    height: u32,
    depth: u32,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let max_extent = width.max(height).max(depth);
    let max_mip_count = u32::BITS - max_extent.leading_zeros();
    if mip_count > max_mip_count {
        return parse_error(
            context,
            format!(
                "{label} mip level count {mip_count} exceeds maximum {max_mip_count} for extent {width}x{height}x{depth}"
            ),
        );
    }
    Ok(())
}

const KTX_CUBEMAP_FACE_COUNT: u32 = 6;

fn read_face_count(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    match read_u32_le(context, offset)? {
        1 => Ok(1),
        KTX_CUBEMAP_FACE_COUNT => Ok(KTX_CUBEMAP_FACE_COUNT),
        value => parse_error(
            context,
            format!("{label} must be 1 for ordinary textures or 6 for cubemaps, got {value}"),
        ),
    }
}

fn ktx_four_byte_padding(byte_len: usize) -> usize {
    (4 - (byte_len % 4)) % 4
}

fn level_index_end(
    context: &AssetImportContext,
    level_count: u32,
) -> Result<u64, AssetImportError> {
    u64::try_from(super::support::KTX2_HEADER_SIZE)
        .ok()
        .and_then(|header_size| {
            u64::from(level_count)
                .checked_mul(u64::try_from(KTX2_LEVEL_INDEX_ENTRY_SIZE).ok()?)
                .and_then(|level_index_len| header_size.checked_add(level_index_len))
        })
        .ok_or_else(|| parse_error_value(context, "ktx2 level index length overflows u64"))
}

fn checked_u64_range_end(
    context: &AssetImportContext,
    label: &str,
    offset: u64,
    length: u64,
) -> Result<u64, AssetImportError> {
    offset
        .checked_add(length)
        .ok_or_else(|| parse_error_value(context, format!("{label} range overflows u64")))
}
