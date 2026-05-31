use super::support::{
    checked_layer_count, parse_error, read_nonzero_u32, read_u32_le, require_len, DDPF_ALPHA,
    DDPF_ALPHAPIXELS, DDPF_BUMPDUDV, DDPF_FOURCC, DDPF_LUMINANCE, DDPF_RGB, DDPF_YUV,
    DDSCAPS2_CUBEMAP, DDSCAPS2_CUBEMAP_ALL_FACES, DDSCAPS2_VOLUME, DDSCAPS_COMPLEX, DDSCAPS_MIPMAP,
    DDSCAPS_TEXTURE, DDSD_CAPS, DDSD_DEPTH, DDSD_HEIGHT, DDSD_LINEARSIZE, DDSD_MIPMAPCOUNT,
    DDSD_PITCH, DDSD_PIXELFORMAT, DDSD_REQUIRED_FLAGS, DDSD_WIDTH, DDS_ALPHA_MODE_CUSTOM,
    DDS_ALPHA_MODE_MASK, DDS_DIMENSION_TEXTURE2D, DDS_RESOURCE_MISC_TEXTURECUBE,
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
    let pitch_or_linear_size = read_u32_le(context, 20)?;
    let raw_depth = read_u32_le(context, 24)?;
    let mip_count = read_mip_count(context, flags)?;
    validate_mip_count_fits_extent(context, width, height, mip_count)?;
    let pixel_format_size = read_u32_le(context, 76)?;
    if pixel_format_size != 32 {
        return parse_error(context, "dds pixel format size must be 32 bytes");
    }

    let pixel_format_flags = read_u32_le(context, 80)?;
    let fourcc = fourcc_string(context, &bytes[84..88])?;
    let rgb_bit_count = read_u32_le(context, 88)?;
    let color_masks = [
        read_u32_le(context, 92)?,
        read_u32_le(context, 96)?,
        read_u32_le(context, 100)?,
        read_u32_le(context, 104)?,
    ];
    validate_pixel_format_flags(
        context,
        pixel_format_flags,
        fourcc.as_deref(),
        rgb_bit_count,
        color_masks,
    )?;
    let caps = read_u32_le(context, 108)?;
    if caps & DDSCAPS_TEXTURE == 0 {
        return parse_error(context, "dds caps must include DDSCAPS_TEXTURE");
    }
    validate_mipmap_caps(context, mip_count, caps)?;
    let caps2 = read_u32_le(context, 112)?;
    let is_legacy_cubemap = caps2 & DDSCAPS2_CUBEMAP != 0;
    validate_cubemap_face_caps(context, caps2, is_legacy_cubemap)?;
    validate_cubemap_caps(context, is_legacy_cubemap, caps)?;
    if is_legacy_cubemap && caps2 & DDSCAPS2_CUBEMAP_ALL_FACES != DDSCAPS2_CUBEMAP_ALL_FACES {
        return parse_error(context, "dds cubemap caps2 must include all six face flags");
    }
    if flags & DDSD_DEPTH != 0 || raw_depth > 0 || caps2 & DDSCAPS2_VOLUME != 0 {
        return parse_error(
            context,
            "dds volume textures are not supported by container importer yet",
        );
    }
    let (format, array_layers, payload_layout) = if fourcc.as_deref() == Some("DX10") {
        require_len(context, DDS_DX10_HEADER_SIZE, "dds dx10 header")?;
        let dxgi_format = read_u32_le(context, 128)?;
        if dxgi_format == 0 {
            return parse_error(context, "dds dx10 format must not be DXGI_FORMAT_UNKNOWN");
        }
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
        let is_dx10_texturecube = misc_flag & DDS_RESOURCE_MISC_TEXTURECUBE != 0;
        if is_legacy_cubemap && is_dx10_texturecube {
            return parse_error(
                context,
                "dds dx10 cubemap must be declared by legacy caps2 or DX10 texturecube flag, not both",
            );
        }
        validate_cubemap_caps(context, is_dx10_texturecube, caps)?;
        let is_cubemap = is_legacy_cubemap || is_dx10_texturecube;
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
            .as_deref()
            .map(|fourcc| format!("dds/{fourcc}"))
            .unwrap_or_else(|| "dds/uncompressed".to_string());
        let layers = if is_legacy_cubemap { 6 } else { 1 };
        let payload_layout = dds_legacy_payload_layout(&format);
        (format, layers, payload_layout)
    };
    validate_pitch_or_linear_size_flags(
        context,
        flags,
        fourcc.as_deref(),
        payload_layout.is_some(),
        pitch_or_linear_size,
    )?;
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
        70..=72 | 79..=81 => DDS_BC1_BYTES_PER_BLOCK,
        73..=78 | 82..=84 | 94..=99 => DDS_BC_FULL_BYTES_PER_BLOCK,
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
    rgb_bit_count: u32,
    color_masks: [u32; 4],
) -> Result<(), AssetImportError> {
    let declares_fourcc = flags & DDPF_FOURCC != 0;
    match (declares_fourcc, fourcc) {
        (true, Some(_)) => Ok(()),
        (true, None) => parse_error(
            context,
            "dds pixel format flags include DDPF_FOURCC but FourCC field is empty",
        ),
        (false, Some(_)) => parse_error(
            context,
            "dds FourCC field is nonzero but DDPF_FOURCC flag is missing",
        ),
        (false, None) => {
            validate_uncompressed_pixel_format_flags(context, flags)?;
            validate_uncompressed_pixel_format_masks(context, flags, rgb_bit_count, color_masks)
        }
    }
}

fn validate_uncompressed_pixel_format_flags(
    context: &AssetImportContext,
    flags: u32,
) -> Result<(), AssetImportError> {
    let primary_layout_count = [
        DDPF_RGB,
        DDPF_YUV,
        DDPF_LUMINANCE,
        DDPF_ALPHA,
        DDPF_BUMPDUDV,
    ]
    .into_iter()
    .filter(|layout_flag| flags & *layout_flag != 0)
    .count();
    if primary_layout_count > 1 {
        return parse_error(
            context,
            "dds uncompressed pixel format flags must declare exactly one primary layout flag",
        );
    }
    if primary_layout_count == 1 {
        return Ok(());
    }
    if flags & DDPF_ALPHAPIXELS != 0 {
        return parse_error(
            context,
            "dds pixel format flags include DDPF_ALPHAPIXELS without a color, luminance, alpha, or bump layout flag",
        );
    }
    parse_error(
        context,
        "dds pixel format flags must declare DDPF_FOURCC or an uncompressed layout flag",
    )
}

fn validate_uncompressed_pixel_format_masks(
    context: &AssetImportContext,
    flags: u32,
    rgb_bit_count: u32,
    color_masks: [u32; 4],
) -> Result<(), AssetImportError> {
    if rgb_bit_count == 0 {
        return parse_error(
            context,
            "dds uncompressed pixel data bit count must be nonzero",
        );
    }
    if rgb_bit_count > u32::BITS {
        return parse_error(
            context,
            "dds uncompressed pixel data bit count must be 1..=32",
        );
    }
    if color_masks.iter().any(|mask| *mask != 0) {
        let allowed_bits = if rgb_bit_count == u32::BITS {
            u32::MAX
        } else {
            (1u32 << rgb_bit_count) - 1
        };
        if !color_masks.iter().all(|mask| *mask & !allowed_bits == 0) {
            return parse_error(
                context,
                "dds uncompressed pixel data channel masks must fit within bit count",
            );
        }
        let mut occupied_bits = 0;
        for mask in color_masks {
            if mask & occupied_bits != 0 {
                return parse_error(
                    context,
                    "dds uncompressed pixel data channel masks must not overlap",
                );
            }
            occupied_bits |= mask;
        }
        if flags & DDPF_ALPHA != 0 && color_masks[3] == 0 {
            return parse_error(
                context,
                "dds DDPF_ALPHA layout must declare a nonzero alpha bit mask",
            );
        }
        if flags & DDPF_ALPHAPIXELS != 0 && color_masks[3] == 0 {
            return parse_error(
                context,
                "dds alpha pixel flags must declare a nonzero alpha bit mask",
            );
        }
        if flags & DDPF_RGB != 0 && !color_masks[..3].iter().any(|mask| *mask != 0) {
            return parse_error(
                context,
                "dds DDPF_RGB layout must declare at least one RGB channel bit mask",
            );
        }
        if flags & DDPF_LUMINANCE != 0 && color_masks[0] == 0 {
            return parse_error(
                context,
                "dds DDPF_LUMINANCE layout must declare a nonzero luminance bit mask",
            );
        }
        if flags & DDPF_YUV != 0 && !color_masks[..3].iter().any(|mask| *mask != 0) {
            return parse_error(
                context,
                "dds DDPF_YUV layout must declare at least one YUV channel bit mask",
            );
        }
        if flags & DDPF_BUMPDUDV != 0 && !color_masks[..3].iter().any(|mask| *mask != 0) {
            return parse_error(
                context,
                "dds DDPF_BUMPDUDV layout must declare at least one bump channel bit mask",
            );
        }
        return Ok(());
    }
    parse_error(
        context,
        "dds uncompressed pixel data must declare at least one channel bit mask",
    )
}

fn validate_pitch_or_linear_size_flags(
    context: &AssetImportContext,
    flags: u32,
    fourcc: Option<&str>,
    has_known_compressed_payload: bool,
    pitch_or_linear_size: u32,
) -> Result<(), AssetImportError> {
    let has_pitch = flags & DDSD_PITCH != 0;
    let has_linear_size = flags & DDSD_LINEARSIZE != 0;
    match (
        has_known_compressed_payload,
        fourcc.is_none(),
        has_pitch,
        has_linear_size,
    ) {
        (true, _, true, true) => parse_error(
            context,
            "dds compressed pixel data must not declare both DDSD_PITCH and DDSD_LINEARSIZE",
        ),
        (true, _, _, false) => parse_error(
            context,
            "dds compressed pixel data must declare DDSD_LINEARSIZE",
        ),
        (true, _, _, true) if pitch_or_linear_size == 0 => parse_error(
            context,
            "dds compressed pixel data linear size must be nonzero",
        ),
        (false, true, true, true) => parse_error(
            context,
            "dds uncompressed pixel data must not declare both DDSD_PITCH and DDSD_LINEARSIZE",
        ),
        (false, true, false, _) => parse_error(
            context,
            "dds uncompressed pixel data must declare DDSD_PITCH",
        ),
        (false, true, true, _) if pitch_or_linear_size == 0 => {
            parse_error(context, "dds uncompressed pixel data pitch must be nonzero")
        }
        _ => Ok(()),
    }
}

fn validate_mipmap_caps(
    context: &AssetImportContext,
    mip_count: u32,
    caps: u32,
) -> Result<(), AssetImportError> {
    if mip_count <= 1 || caps & DDSCAPS_MIPMAP != 0 && caps & DDSCAPS_COMPLEX != 0 {
        return Ok(());
    }
    parse_error(
        context,
        "dds caps must include DDSCAPS_MIPMAP and DDSCAPS_COMPLEX when multiple mip levels are declared",
    )
}

fn validate_cubemap_caps(
    context: &AssetImportContext,
    is_cubemap: bool,
    caps: u32,
) -> Result<(), AssetImportError> {
    if !is_cubemap || caps & DDSCAPS_COMPLEX != 0 {
        return Ok(());
    }
    parse_error(
        context,
        "dds caps must include DDSCAPS_COMPLEX when cubemap faces are declared or DX10 texturecube flag is set",
    )
}

fn validate_cubemap_face_caps(
    context: &AssetImportContext,
    caps2: u32,
    is_cubemap: bool,
) -> Result<(), AssetImportError> {
    let face_flags = DDSCAPS2_CUBEMAP_ALL_FACES & !DDSCAPS2_CUBEMAP;
    if is_cubemap || caps2 & face_flags == 0 {
        return Ok(());
    }
    parse_error(
        context,
        "dds cubemap face caps2 flags require DDSCAPS2_CUBEMAP",
    )
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
