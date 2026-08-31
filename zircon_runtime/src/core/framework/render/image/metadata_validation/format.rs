use super::super::TextureCompressionTarget;

pub(super) fn format_has_srgb_variant(format: &str) -> bool {
    let format = format.trim().to_ascii_lowercase();
    if format.starts_with("astc/") {
        return astc_2d_format_has_srgb_variant(&format);
    }
    if format.starts_with("dds/") {
        return dds_format_has_srgb_variant(&format);
    }
    if format.starts_with("ktx/") {
        return ktx_gl_format_has_srgb_variant(&format);
    }
    if format.starts_with("ktx2/") {
        return ktx2_format_has_srgb_variant(&format);
    }

    matches!(
        format.as_str(),
        "rgba8unorm"
            | "rgba8unorm_srgb"
            | "bgra8unorm"
            | "bgra8unorm_srgb"
            | "bc1"
            | "bc2"
            | "bc3"
            | "bc7"
    )
}

fn dds_format_has_srgb_variant(format: &str) -> bool {
    matches!(
        format,
        "dds/dxt1"
            | "dds/dxt3"
            | "dds/dxt5"
            | "dds/dxgi-72"
            | "dds/dxgi-75"
            | "dds/dxgi-78"
            | "dds/dxgi-99"
    )
}

fn astc_2d_format_has_srgb_variant(format: &str) -> bool {
    let Some(dimensions) = format.strip_prefix("astc/") else {
        return false;
    };
    let mut parts = dimensions.split('x');
    let (Some(width), Some(height), Some(depth)) = (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    let (Ok(width), Ok(height), Ok(depth)) = (
        width.parse::<u32>(),
        height.parse::<u32>(),
        depth.parse::<u32>(),
    ) else {
        return false;
    };

    depth == 1
        && matches!(
            (width, height),
            (4, 4)
                | (5, 4)
                | (5, 5)
                | (6, 5)
                | (6, 6)
                | (8, 5)
                | (8, 6)
                | (8, 8)
                | (10, 5)
                | (10, 6)
                | (10, 8)
                | (10, 10)
                | (12, 10)
                | (12, 12)
        )
}

fn ktx_gl_format_has_srgb_variant(format: &str) -> bool {
    matches!(
        ktx_gl_internal_format(format),
        Some(
            0x8c4c | 0x8c4d | 0x8c4e | 0x8c4f | 0x8e8d | 0x9275 | 0x9277 | 0x9279 | 0x93d0..=0x93dd
        )
    )
}

fn ktx2_format_has_srgb_variant(format: &str) -> bool {
    let Some(vk_format) = ktx2_vk_format(format) else {
        return false;
    };
    matches!(
        vk_format,
        132 | 134 | 136 | 138 | 146 | 148 | 150 | 152 | 158..=184 if vk_format % 2 == 0
    )
}

pub(super) fn format_is_float_family(format: &str) -> bool {
    let format = format.trim().to_ascii_lowercase();
    if format.starts_with("astc/") {
        return false;
    }
    if format.starts_with("dds/") {
        return matches!(format.as_str(), "dds/dxgi-95" | "dds/dxgi-96");
    }
    if format.starts_with("ktx/") {
        return matches!(ktx_gl_internal_format(&format), Some(0x8e8e | 0x8e8f));
    }
    if format.starts_with("ktx2/") {
        return matches!(ktx2_vk_format(&format), Some(143 | 144));
    }

    matches!(
        format.as_str(),
        "r16float"
            | "r32float"
            | "rg16float"
            | "rg32float"
            | "rgba16float"
            | "rgba32float"
            | "rg11b10ufloat"
            | "rgb9e5ufloat"
            | "bc6h"
    )
}

fn ktx_gl_internal_format(format: &str) -> Option<u32> {
    format
        .strip_prefix("ktx/gl-internal-0x")
        .and_then(|value| u32::from_str_radix(value, 16).ok())
}

fn ktx2_vk_format(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("vk-"))
        .and_then(|value| value.parse().ok())
}

pub(super) fn format_supports_runtime_mip_generation(format: &str) -> bool {
    matches!(
        format.trim().to_ascii_lowercase().as_str(),
        "rgba8unorm" | "rgba8unorm_srgb"
    )
}

pub(super) fn compression_name(compression: TextureCompressionTarget) -> &'static str {
    match compression {
        TextureCompressionTarget::Auto => "auto",
        TextureCompressionTarget::Uncompressed => "uncompressed",
        TextureCompressionTarget::Bc1 => "bc1",
        TextureCompressionTarget::Bc4 => "bc4",
        TextureCompressionTarget::Bc5 => "bc5",
        TextureCompressionTarget::Bc6h => "bc6h",
        TextureCompressionTarget::Bc7 => "bc7",
    }
}
