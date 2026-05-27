use super::support::{
    checked_layer_count, parse_error, read_nonzero_u32, read_u32_le, require_len, DDPF_FOURCC,
    DDSCAPS2_CUBEMAP, DDSCAPS2_CUBEMAP_ALL_FACES, DDSCAPS2_VOLUME, DDSCAPS_TEXTURE, DDSD_CAPS,
    DDSD_HEIGHT, DDSD_MIPMAPCOUNT, DDSD_PIXELFORMAT, DDSD_REQUIRED_FLAGS, DDSD_WIDTH,
    DDS_ALPHA_MODE_CUSTOM, DDS_ALPHA_MODE_MASK, DDS_DIMENSION_TEXTURE2D,
    DDS_RESOURCE_MISC_TEXTURECUBE,
};
use super::TextureContainerInfo;
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const DDS_BLOCK_WIDTH: usize = 4;
const DDS_BLOCK_HEIGHT: usize = 4;
const DDS_BC1_BYTES_PER_BLOCK: usize = 8;
const DDS_BC_FULL_BYTES_PER_BLOCK: usize = 16;

pub(super) fn parse(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, DDS_HEADER_SIZE, "dds header")?;
    if &bytes[..4] != b"DDS " {
        return parse_error(context, "dds header missing DDS magic");
    }
    if read_u32_le(context, 4)? != 124 {
        return parse_error(context, "dds header size must be 124 bytes");
    }
    let flags = read_u32_le(context, 8)?;
    validate_required_flags(context, flags)?;

    let height = read_nonzero_u32(context, 12, "dds height")?;
    let width = read_nonzero_u32(context, 16, "dds width")?;
    let raw_depth = read_u32_le(context, 24)?;
    let mip_count = read_mip_count(context, flags)?;
    validate_mip_count_fits_extent(context, width, height, mip_count)?;
    let pixel_format_size = read_u32_le(context, 76)?;
    if pixel_format_size != 32 {
        return parse_error(context, "dds pixel format size must be 32 bytes");
    }

    let pixel_format_flags = read_u32_le(context, 80)?;
    let fourcc = fourcc_string(context, &bytes[84..88])?;
    validate_pixel_format_flags(context, pixel_format_flags, fourcc.as_deref())?;
    let caps = read_u32_le(context, 108)?;
    if caps & DDSCAPS_TEXTURE == 0 {
        return parse_error(context, "dds caps must include DDSCAPS_TEXTURE");
    }
    let caps2 = read_u32_le(context, 112)?;
    let is_legacy_cubemap = caps2 & DDSCAPS2_CUBEMAP != 0;
    if is_legacy_cubemap && caps2 & DDSCAPS2_CUBEMAP_ALL_FACES != DDSCAPS2_CUBEMAP_ALL_FACES {
        return parse_error(context, "dds cubemap caps2 must include all six face flags");
    }
    if raw_depth > 0 || caps2 & DDSCAPS2_VOLUME != 0 {
        return parse_error(
            context,
            "dds volume textures are not supported by container importer yet",
        );
    }
    let (format, array_layers, payload_layout) = if fourcc.as_deref() == Some("DX10") {
        require_len(context, DDS_DX10_HEADER_SIZE, "dds dx10 header")?;
        let dxgi_format = read_u32_le(context, 128)?;
        let resource_dimension = read_u32_le(context, 132)?;
        if resource_dimension != DDS_DIMENSION_TEXTURE2D {
            return parse_error(
                context,
                format!("dds dx10 resource dimension must be texture2d, got {resource_dimension}"),
            );
        }
        let misc_flag = read_u32_le(context, 136)?;
        let unsupported_misc_flag_bits = misc_flag & !DDS_RESOURCE_MISC_TEXTURECUBE;
        if unsupported_misc_flag_bits != 0 {
            return parse_error(
                context,
                format!(
                    "dds dx10 misc flag contains unsupported bits 0x{unsupported_misc_flag_bits:08x}"
                ),
            );
        }
        let is_cubemap = is_legacy_cubemap || misc_flag & DDS_RESOURCE_MISC_TEXTURECUBE != 0;
        let array_size = read_nonzero_u32(context, 140, "dds dx10 array size")?;
        validate_dx10_misc_flags2(context, read_u32_le(context, 144)?)?;
        let faces = if is_cubemap { 6 } else { 1 };
        let array_layers =
            checked_layer_count(context, "dds dx10 array layer count", array_size, faces)?;
        (
            format!("dds/dxgi-{dxgi_format}"),
            array_layers,
            dds_dx10_payload_layout(dxgi_format),
        )
    } else {
        let format = fourcc
            .map(|fourcc| format!("dds/{fourcc}"))
            .unwrap_or_else(|| "dds/uncompressed".to_string());
        let layers = if is_legacy_cubemap { 6 } else { 1 };
        let payload_layout = dds_legacy_payload_layout(&format);
        (format, layers, payload_layout)
    };
    if let Some(payload_layout) = payload_layout {
        validate_compressed_payload_len(
            context,
            width,
            height,
            array_layers,
            mip_count,
            payload_layout,
        )?;
    }

    Ok(TextureContainerInfo {
        format,
        width,
        height,
        dimension: RenderImageDimension::D2,
        depth_or_array_layers: array_layers,
        mip_count,
        array_layers,
    })
}

#[derive(Clone, Copy)]
struct DdsCompressedPayloadLayout {
    data_offset: usize,
    bytes_per_block: usize,
}

fn validate_compressed_payload_len(
    context: &AssetImportContext,
    width: u32,
    height: u32,
    array_layers: u32,
    mip_count: u32,
    layout: DdsCompressedPayloadLayout,
) -> Result<(), AssetImportError> {
    let required_payload_len =
        dds_mip_chain_payload_len(context, width, height, array_layers, mip_count, layout)?;
    let required_len = layout
        .data_offset
        .checked_add(required_payload_len)
        .ok_or_else(|| dds_payload_range_error(context))?;
    require_len(context, required_len, "dds compressed mip chain payload")
}

fn dds_mip_chain_payload_len(
    context: &AssetImportContext,
    width: u32,
    height: u32,
    array_layers: u32,
    mip_count: u32,
    layout: DdsCompressedPayloadLayout,
) -> Result<usize, AssetImportError> {
    let layers = usize::try_from(array_layers).expect("u32 layer count fits usize");
    let mut mip_width = usize::try_from(width).expect("u32 width fits usize");
    let mut mip_height = usize::try_from(height).expect("u32 height fits usize");
    let mut total_len = 0_usize;
    for _ in 0..mip_count {
        let mip_len = dds_mip_level_payload_len(context, mip_width, mip_height, layers, layout)?;
        total_len = total_len
            .checked_add(mip_len)
            .ok_or_else(|| dds_payload_range_error(context))?;
        mip_width = (mip_width / 2).max(1);
        mip_height = (mip_height / 2).max(1);
    }
    Ok(total_len)
}

fn dds_mip_level_payload_len(
    context: &AssetImportContext,
    width: usize,
    height: usize,
    layers: usize,
    layout: DdsCompressedPayloadLayout,
) -> Result<usize, AssetImportError> {
    let block_columns = width.div_ceil(DDS_BLOCK_WIDTH);
    let block_rows = height.div_ceil(DDS_BLOCK_HEIGHT);
    block_columns
        .checked_mul(block_rows)
        .and_then(|blocks| blocks.checked_mul(layers))
        .and_then(|blocks| blocks.checked_mul(layout.bytes_per_block))
        .ok_or_else(|| dds_payload_range_error(context))
}

fn dds_legacy_payload_layout(format: &str) -> Option<DdsCompressedPayloadLayout> {
    let bytes_per_block = match format.trim().to_ascii_lowercase().as_str() {
        "dds/dxt1" | "dds/ati1" | "dds/bc4u" | "dds/bc4s" => DDS_BC1_BYTES_PER_BLOCK,
        "dds/dxt3" | "dds/dxt5" | "dds/ati2" | "dds/bc5u" | "dds/bc5s" => {
            DDS_BC_FULL_BYTES_PER_BLOCK
        }
        _ => return None,
    };
    Some(DdsCompressedPayloadLayout {
        data_offset: DDS_HEADER_SIZE,
        bytes_per_block,
    })
}

fn dds_dx10_payload_layout(dxgi_format: u32) -> Option<DdsCompressedPayloadLayout> {
    let bytes_per_block = match dxgi_format {
        71..=74 | 80 | 81 => DDS_BC1_BYTES_PER_BLOCK,
        75..=79 | 82..=84 | 95..=99 => DDS_BC_FULL_BYTES_PER_BLOCK,
        _ => return None,
    };
    Some(DdsCompressedPayloadLayout {
        data_offset: DDS_DX10_HEADER_SIZE,
        bytes_per_block,
    })
}

fn dds_payload_range_error(context: &AssetImportContext) -> AssetImportError {
    super::support::parse_error_value(
        context,
        "dds compressed mip chain payload range overflows usize",
    )
}

fn validate_dx10_misc_flags2(
    context: &AssetImportContext,
    misc_flags2: u32,
) -> Result<(), AssetImportError> {
    let reserved_bits = misc_flags2 & !DDS_ALPHA_MODE_MASK;
    if reserved_bits != 0 {
        return parse_error(
            context,
            format!("dds dx10 miscFlags2 contains reserved bits 0x{reserved_bits:08x}"),
        );
    }
    let alpha_mode = misc_flags2 & DDS_ALPHA_MODE_MASK;
    if alpha_mode > DDS_ALPHA_MODE_CUSTOM {
        return parse_error(
            context,
            format!("dds dx10 alpha mode must be 0..=4, got {alpha_mode}"),
        );
    }
    Ok(())
}

fn validate_pixel_format_flags(
    context: &AssetImportContext,
    flags: u32,
    fourcc: Option<&str>,
) -> Result<(), AssetImportError> {
    let declares_fourcc = flags & DDPF_FOURCC != 0;
    match (declares_fourcc, fourcc) {
        (true, Some(_)) | (false, None) => Ok(()),
        (true, None) => parse_error(
            context,
            "dds pixel format flags include DDPF_FOURCC but FourCC field is empty",
        ),
        (false, Some(_)) => parse_error(
            context,
            "dds FourCC field is nonzero but DDPF_FOURCC flag is missing",
        ),
    }
}

fn validate_mip_count_fits_extent(
    context: &AssetImportContext,
    width: u32,
    height: u32,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let max_mip_count = u32::BITS - width.max(height).leading_zeros();
    if mip_count > max_mip_count {
        return parse_error(
            context,
            format!(
                "dds mip map count {mip_count} exceeds maximum {max_mip_count} for extent {width}x{height}"
            ),
        );
    }
    Ok(())
}

fn validate_required_flags(
    context: &AssetImportContext,
    flags: u32,
) -> Result<(), AssetImportError> {
    if flags & DDSD_REQUIRED_FLAGS == DDSD_REQUIRED_FLAGS {
        return Ok(());
    }

    let required = [
        (DDSD_CAPS, "DDSD_CAPS"),
        (DDSD_HEIGHT, "DDSD_HEIGHT"),
        (DDSD_WIDTH, "DDSD_WIDTH"),
        (DDSD_PIXELFORMAT, "DDSD_PIXELFORMAT"),
    ];
    for (flag, label) in required {
        if flags & flag == 0 {
            return parse_error(context, format!("dds header flags must include {label}"));
        }
    }
    Ok(())
}

fn read_mip_count(context: &AssetImportContext, flags: u32) -> Result<u32, AssetImportError> {
    if flags & DDSD_MIPMAPCOUNT == 0 {
        return Ok(1);
    }
    read_nonzero_u32(
        context,
        28,
        "dds mip map count when DDSD_MIPMAPCOUNT is set",
    )
}

fn fourcc_string(
    context: &AssetImportContext,
    bytes: &[u8],
) -> Result<Option<String>, AssetImportError> {
    if bytes.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    if bytes.contains(&0) {
        return parse_error(context, "dds fourcc must not contain embedded NUL bytes");
    }
    if !bytes
        .iter()
        .all(|byte| byte.is_ascii() && !byte.is_ascii_control())
    {
        return parse_error(context, "dds fourcc must contain printable ASCII bytes");
    }
    let fourcc =
        std::str::from_utf8(bytes).expect("DDS FourCC bytes were checked as printable ASCII");
    Ok(Some(fourcc.to_string()))
}
