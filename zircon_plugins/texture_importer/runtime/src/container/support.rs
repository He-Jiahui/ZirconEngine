use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

pub(super) fn texture_depth_or_array_layers(
    dimension: RenderImageDimension,
    depth: u32,
    array_layers: u32,
) -> u32 {
    if dimension == RenderImageDimension::D3 {
        depth.max(1)
    } else {
        array_layers.max(1)
    }
}

pub(super) fn checked_layer_count(
    context: &AssetImportContext,
    label: &str,
    layers: u32,
    faces: u32,
) -> Result<u32, AssetImportError> {
    layers
        .checked_mul(faces)
        .filter(|value| *value > 0)
        .ok_or_else(|| parse_error_value(context, format!("{label} overflows u32")))
}

pub(super) fn require_len(
    context: &AssetImportContext,
    required: usize,
    label: &str,
) -> Result<(), AssetImportError> {
    if context.source_bytes.len() < required {
        return parse_error(
            context,
            format!(
                "{label} requires at least {required} bytes, got {}",
                context.source_bytes.len()
            ),
        );
    }
    Ok(())
}

pub(super) fn read_nonzero_u32(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    let value = read_u32_le(context, offset)?;
    if value == 0 {
        return parse_error(context, format!("{label} must be nonzero"));
    }
    Ok(value)
}

pub(super) fn read_nonzero_u8(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u8, AssetImportError> {
    let value = context.source_bytes[offset];
    if value == 0 {
        return parse_error(context, format!("{label} must be nonzero"));
    }
    Ok(value)
}

pub(super) fn read_nonzero_u24_le(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    let value = read_u24_le(&context.source_bytes, offset);
    if value == 0 {
        return parse_error(context, format!("{label} must be nonzero"));
    }
    Ok(value)
}

pub(super) fn read_u32_le(
    context: &AssetImportContext,
    offset: usize,
) -> Result<u32, AssetImportError> {
    let bytes = context
        .source_bytes
        .get(offset..offset + 4)
        .ok_or_else(|| parse_error_value(context, format!("missing u32 at byte {offset}")))?;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("slice length checked before conversion"),
    ))
}

pub(super) fn read_u64_le(
    context: &AssetImportContext,
    offset: usize,
) -> Result<u64, AssetImportError> {
    let bytes = context
        .source_bytes
        .get(offset..offset + 8)
        .ok_or_else(|| parse_error_value(context, format!("missing u64 at byte {offset}")))?;
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .expect("slice length checked before conversion"),
    ))
}

fn read_u24_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from(bytes[offset])
        | (u32::from(bytes[offset + 1]) << 8)
        | (u32::from(bytes[offset + 2]) << 16)
}

pub(super) fn parse_error<T>(
    context: &AssetImportContext,
    message: impl Into<String>,
) -> Result<T, AssetImportError> {
    Err(parse_error_value(context, message))
}

pub(super) fn parse_error_value(
    context: &AssetImportContext,
    message: impl Into<String>,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "parse texture container {}: {}",
        context.source_path.display(),
        message.into()
    ))
}

pub(super) const DDSD_CAPS: u32 = 0x0000_0001;
pub(super) const DDSD_HEIGHT: u32 = 0x0000_0002;
pub(super) const DDSD_WIDTH: u32 = 0x0000_0004;
pub(super) const DDSD_PITCH: u32 = 0x0000_0008;
pub(super) const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
pub(super) const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
pub(super) const DDSD_LINEARSIZE: u32 = 0x0008_0000;
pub(super) const DDSD_DEPTH: u32 = 0x0080_0000;
pub(super) const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
pub(super) const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
pub(super) const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
pub(super) const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
pub(super) const DDSCAPS2_CUBEMAP: u32 = 0x0000_0200;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEX: u32 = 0x0000_0400;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEX: u32 = 0x0000_0800;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEY: u32 = 0x0000_1000;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEY: u32 = 0x0000_2000;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEZ: u32 = 0x0000_4000;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEZ: u32 = 0x0000_8000;
pub(super) const DDSCAPS2_VOLUME: u32 = 0x0020_0000;
pub(super) const DDPF_ALPHAPIXELS: u32 = 0x0000_0001;
pub(super) const DDPF_ALPHA: u32 = 0x0000_0002;
pub(super) const DDPF_FOURCC: u32 = 0x0000_0004;
pub(super) const DDPF_RGB: u32 = 0x0000_0040;
pub(super) const DDPF_YUV: u32 = 0x0000_0200;
pub(super) const DDPF_LUMINANCE: u32 = 0x0002_0000;
pub(super) const DDPF_BUMPDUDV: u32 = 0x0008_0000;
pub(super) const DDSCAPS2_CUBEMAP_ALL_FACES: u32 = DDSCAPS2_CUBEMAP
    | DDSCAPS2_CUBEMAP_POSITIVEX
    | DDSCAPS2_CUBEMAP_NEGATIVEX
    | DDSCAPS2_CUBEMAP_POSITIVEY
    | DDSCAPS2_CUBEMAP_NEGATIVEY
    | DDSCAPS2_CUBEMAP_POSITIVEZ
    | DDSCAPS2_CUBEMAP_NEGATIVEZ;
pub(super) const DDS_DIMENSION_TEXTURE2D: u32 = 3;
pub(super) const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
pub(super) const DDS_ALPHA_MODE_MASK: u32 = 0x7;
pub(super) const DDS_ALPHA_MODE_CUSTOM: u32 = 0x4;
pub(super) const KTX_LITTLE_ENDIAN: u32 = 0x0403_0201;
pub(super) const KTX1_IDENTIFIER: &[u8] = b"\xABKTX 11\xBB\r\n\x1A\n";
pub(super) const KTX2_IDENTIFIER: &[u8] = b"\xABKTX 20\xBB\r\n\x1A\n";
pub(super) const KTX2_HEADER_SIZE: usize = 80;
pub(super) const KTX2_LEVEL_INDEX_ENTRY_SIZE: usize = 24;
pub(super) const KTX2_SUPERCOMPRESSION_NONE: u32 = 0;
pub(super) const KTX2_SUPERCOMPRESSION_BASIS_LZ: u32 = 1;
pub(super) const KTX2_SUPERCOMPRESSION_ZSTANDARD: u32 = 2;
pub(super) const KTX2_SUPERCOMPRESSION_ZLIB: u32 = 3;
pub(super) const ASTC_MAGIC: &[u8] = b"\x13\xAB\xA1\x5C";
