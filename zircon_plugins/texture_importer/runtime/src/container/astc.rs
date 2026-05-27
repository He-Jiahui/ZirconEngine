use super::{
    support::{
        parse_error, read_nonzero_u24_le, read_nonzero_u8, require_len,
        texture_depth_or_array_layers, ASTC_MAGIC,
    },
    TextureContainerInfo,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

const ASTC_HEADER_SIZE: usize = 16;
const ASTC_BYTES_PER_BLOCK: usize = 16;

pub(super) fn parse(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, ASTC_HEADER_SIZE, "astc header")?;
    if &bytes[..4] != ASTC_MAGIC {
        return parse_error(context, "astc header missing ASTC magic");
    }

    let block_x = read_nonzero_u8(context, 4, "astc block x")?;
    let block_y = read_nonzero_u8(context, 5, "astc block y")?;
    let block_z = read_nonzero_u8(context, 6, "astc block z")?;
    let width = read_nonzero_u24_le(context, 7, "astc width")?;
    let height = read_nonzero_u24_le(context, 10, "astc height")?;
    let depth = read_nonzero_u24_le(context, 13, "astc depth")?;
    let payload_len = astc_payload_len(context, block_x, block_y, block_z, width, height, depth)?;
    let required_len = ASTC_HEADER_SIZE
        .checked_add(payload_len)
        .ok_or_else(|| astc_payload_range_error(context))?;
    require_len(context, required_len, "astc block payload")?;
    let dimension = if depth > 1 || block_z > 1 {
        RenderImageDimension::D3
    } else {
        RenderImageDimension::D2
    };

    Ok(TextureContainerInfo {
        format: format!("astc/{block_x}x{block_y}x{block_z}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, depth, depth),
        mip_count: 1,
        array_layers: array_layers(dimension, depth),
    })
}

fn astc_payload_len(
    context: &AssetImportContext,
    block_x: u8,
    block_y: u8,
    block_z: u8,
    width: u32,
    height: u32,
    depth: u32,
) -> Result<usize, AssetImportError> {
    let blocks_x = div_ceil_checked(
        usize::try_from(width).expect("u32 width fits usize"),
        block_x,
    );
    let blocks_y = div_ceil_checked(
        usize::try_from(height).expect("u32 height fits usize"),
        block_y,
    );
    let blocks_z = div_ceil_checked(
        usize::try_from(depth).expect("u32 depth fits usize"),
        block_z,
    );
    blocks_x
        .checked_mul(blocks_y)
        .and_then(|blocks| blocks.checked_mul(blocks_z))
        .and_then(|blocks| blocks.checked_mul(ASTC_BYTES_PER_BLOCK))
        .ok_or_else(|| astc_payload_range_error(context))
}

fn div_ceil_checked(value: usize, divisor: u8) -> usize {
    let divisor = usize::from(divisor);
    value.div_ceil(divisor)
}

fn astc_payload_range_error(context: &AssetImportContext) -> AssetImportError {
    super::support::parse_error_value(context, "astc block payload range overflows usize")
}

fn array_layers(dimension: RenderImageDimension, depth: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        depth.max(1)
    }
}
