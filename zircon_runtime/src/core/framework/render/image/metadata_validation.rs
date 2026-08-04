use super::{
    RenderImageColorSpace, RenderSamplerDescriptor, RenderSamplerFilter, TextureCompressionTarget,
    TextureMetadata, TextureMipFilter, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureMetadataDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureMetadataDiagnostic {
    pub severity: TextureMetadataDiagnosticSeverity,
    pub message: String,
}

pub fn validate_texture_metadata(
    uri: &str,
    format: &str,
    metadata: &TextureMetadata,
    sampler: &RenderSamplerDescriptor,
) -> Vec<TextureMetadataDiagnostic> {
    let mut diagnostics = Vec::new();
    if metadata.usage_hint == TextureUsageHint::Normal
        && metadata.color_space == RenderImageColorSpace::Srgb
    {
        error(
            &mut diagnostics,
            format!("normal map must use linear color space: '{uri}' declares color_space=srgb"),
        );
    }
    if metadata.usage_hint == TextureUsageHint::Albedo
        && metadata.color_space == RenderImageColorSpace::Linear
    {
        warning(
            &mut diagnostics,
            format!("albedo texture '{uri}' declares linear; expected srgb unless intentional"),
        );
    }
    if metadata.usage_hint == TextureUsageHint::Hdr && !format_is_float_family(format) {
        error(
            &mut diagnostics,
            format!("hdr texture '{uri}' requires a float format, got '{format}'"),
        );
    }
    if metadata.usage_hint == TextureUsageHint::Normal
        && !matches!(
            metadata.compression,
            TextureCompressionTarget::Auto
                | TextureCompressionTarget::Uncompressed
                | TextureCompressionTarget::Bc5
        )
    {
        warning(
            &mut diagnostics,
            format!(
                "normal map '{uri}' should compress as bc5, got '{}'",
                compression_name(metadata.compression)
            ),
        );
    }
    if metadata.normal_convention != TextureNormalConvention::None
        && metadata.usage_hint != TextureUsageHint::Normal
    {
        error(
            &mut diagnostics,
            format!("normal_convention is only valid for usage_hint=normal: '{uri}'"),
        );
    }
    if metadata.compression == TextureCompressionTarget::Bc6h
        && metadata.color_space == RenderImageColorSpace::Srgb
    {
        error(
            &mut diagnostics,
            format!("bc6h has no srgb variant: '{uri}'"),
        );
    }
    if metadata.svt.is_some() && metadata.mip_policy == TextureMipPolicy::None {
        error(
            &mut diagnostics,
            format!("svt texture '{uri}' requires a full mip chain for its mip tail"),
        );
    }
    if metadata.usage_hint == TextureUsageHint::Ui && metadata.mip_policy != TextureMipPolicy::None
    {
        warning(
            &mut diagnostics,
            format!("ui texture '{uri}' rarely needs mips; consider mip_policy=none"),
        );
    }
    if metadata.mip_policy == TextureMipPolicy::None
        && sampler.mipmap_filter == RenderSamplerFilter::Linear
    {
        warning(
            &mut diagnostics,
            format!("'{uri}' samples with trilinear filter but declares mip_policy=none"),
        );
    }
    if metadata.mip_policy == TextureMipPolicy::GenerateRuntime
        && metadata.mip_filter != TextureMipFilter::Box
    {
        error(
            &mut diagnostics,
            format!("runtime mip generation requires mip_filter=box: '{uri}'"),
        );
    }
    if metadata.mip_policy == TextureMipPolicy::GenerateRuntime
        && !format_supports_runtime_mip_generation(format)
    {
        error(
            &mut diagnostics,
            format!(
                "runtime mip generation supports only rgba8unorm storage: '{uri}' declares format '{format}'"
            ),
        );
    }
    if metadata.color_space == RenderImageColorSpace::Srgb && !format_has_srgb_variant(format) {
        error(
            &mut diagnostics,
            format!("format '{format}' has no srgb variant: '{uri}'"),
        );
    }
    if !matches!(metadata.max_anisotropy, 1 | 2 | 4 | 8 | 16) {
        error(
            &mut diagnostics,
            format!(
                "max_anisotropy must be one of 1, 2, 4, 8, or 16: '{uri}' declares {}",
                metadata.max_anisotropy
            ),
        );
    }
    if metadata.max_anisotropy > 1
        && (sampler.mag_filter != RenderSamplerFilter::Linear
            || sampler.min_filter != RenderSamplerFilter::Linear
            || sampler.mipmap_filter != RenderSamplerFilter::Linear)
    {
        error(
            &mut diagnostics,
            format!("max_anisotropy requires linear mag/min/mipmap filters: '{uri}'"),
        );
    }
    if !metadata.mip_bias.is_finite() {
        error(
            &mut diagnostics,
            format!("mip_bias must be finite: '{uri}'"),
        );
    }
    diagnostics
}

fn format_has_srgb_variant(format: &str) -> bool {
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

fn format_is_float_family(format: &str) -> bool {
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

fn format_supports_runtime_mip_generation(format: &str) -> bool {
    matches!(
        format.trim().to_ascii_lowercase().as_str(),
        "rgba8unorm" | "rgba8unorm_srgb"
    )
}

fn compression_name(compression: TextureCompressionTarget) -> &'static str {
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

fn error(diagnostics: &mut Vec<TextureMetadataDiagnostic>, message: String) {
    diagnostics.push(TextureMetadataDiagnostic {
        severity: TextureMetadataDiagnosticSeverity::Error,
        message,
    });
}

fn warning(diagnostics: &mut Vec<TextureMetadataDiagnostic>, message: String) {
    diagnostics.push(TextureMetadataDiagnostic {
        severity: TextureMetadataDiagnosticSeverity::Warning,
        message,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_srgb_metadata_is_an_error() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/normal.png",
            "rgba8unorm_srgb",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
    }

    #[test]
    fn albedo_linear_metadata_is_a_warning() {
        let metadata = TextureMetadata {
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/albedo.png",
            "rgba8unorm",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning
                && diagnostic.message
                    == "albedo texture 'textures/albedo.png' declares linear; expected srgb unless intentional"
        }));
    }

    #[test]
    fn hdr_metadata_requires_a_float_format() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/sky.png",
            "rgba8unorm",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message
                    == "hdr texture 'textures/sky.png' requires a float format, got 'rgba8unorm'"
        }));
    }

    #[test]
    fn hdr_metadata_accepts_float_format() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/sky.hdr",
            "rgba16float",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(!diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
    }

    #[test]
    fn hdr_metadata_accepts_bc6h_compressed_container_formats() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        for format in [
            "dds/dxgi-95",
            "dds/dxgi-96",
            "ktx/gl-internal-0x8e8e",
            "ktx/gl-internal-0x8e8f",
            "ktx2/vk-143",
            "ktx2/vk-144",
        ] {
            let diagnostics = validate_texture_metadata(
                "textures/environment.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(
                !diagnostics.iter().any(|diagnostic| {
                    diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                }),
                "{format} should support HDR metadata: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn hdr_metadata_rejects_non_float_compressed_container_formats() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        for format in ["dds/dxt1", "ktx/gl-internal-0x83f0", "ktx2/vk-131"] {
            let diagnostics = validate_texture_metadata(
                "textures/invalid-environment.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("requires a float format")
            }));
        }
    }

    #[test]
    fn hdr_metadata_rejects_container_tokens_with_spurious_float_suffixes() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        for format in [
            "dds/dxgi-71-float",
            "ktx/gl-internal-0x83f0-float",
            "ktx2/vk-131-float",
            "astc/4x4x1-float",
        ] {
            let diagnostics = validate_texture_metadata(
                "textures/invalid-hdr-container.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("requires a float format")
            }));
        }
    }

    #[test]
    fn hdr_metadata_rejects_unrecognized_float_format_tokens() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        };
        for format in [
            "rgba16float-srgb",
            "unrecognized-float-format",
            "bc6h-untrusted",
        ] {
            let diagnostics = validate_texture_metadata(
                "textures/invalid-hdr-format.texture",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("requires a float format")
            }));
        }
    }

    #[test]
    fn normal_non_bc5_compression_is_a_warning() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            color_space: RenderImageColorSpace::Linear,
            compression: TextureCompressionTarget::Bc7,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/normal.png",
            "rgba8unorm",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning
                && diagnostic.message
                    == "normal map 'textures/normal.png' should compress as bc5, got 'bc7'"
        }));
    }

    #[test]
    fn ui_mip_policy_is_a_warning() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Ui,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "ui/icon.png",
            "rgba8unorm_srgb",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning));
    }

    #[test]
    fn srgb_metadata_accepts_supported_compressed_container_formats() {
        let metadata = TextureMetadata::default();
        for format in [
            "dds/dxt1",
            "dds/dxt3",
            "dds/dxt5",
            "astc/4x4x1",
            "ktx/gl-internal-0x8c4c",
            "ktx/gl-internal-0x8e8d",
            "ktx2/vk-132",
        ] {
            let diagnostics = validate_texture_metadata(
                "textures/albedo.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(
                !diagnostics.iter().any(|diagnostic| {
                    diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                }),
                "{format} should support sRGB metadata: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn srgb_metadata_rejects_linear_compressed_container_formats() {
        let metadata = TextureMetadata::default();
        for format in ["dds/dxgi-71", "ktx/gl-internal-0x83f0", "ktx2/vk-131"] {
            let diagnostics = validate_texture_metadata(
                "textures/linear.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("has no srgb variant")
            }));
        }
    }

    #[test]
    fn srgb_metadata_rejects_container_tokens_with_spurious_srgb_suffixes() {
        let metadata = TextureMetadata::default();
        for format in [
            "dds/dxgi-71-srgb",
            "ktx/gl-internal-0x83f0-srgb",
            "ktx2/vk-131-srgb",
        ] {
            let diagnostics = validate_texture_metadata(
                "textures/invalid-srgb-container.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("has no srgb variant")
            }));
        }
    }

    #[test]
    fn srgb_metadata_rejects_unsupported_astc_container_formats() {
        let metadata = TextureMetadata::default();
        for format in ["astc/4x4", "astc/4x4x2", "astc/7x7x1"] {
            let diagnostics = validate_texture_metadata(
                "textures/unsupported-astc.container",
                format,
                &metadata,
                &RenderSamplerDescriptor::default(),
            );

            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                    && diagnostic.message.contains("has no srgb variant")
            }));
        }
    }

    #[test]
    fn runtime_mip_policy_requires_box_filter() {
        let metadata = TextureMetadata {
            mip_policy: TextureMipPolicy::GenerateRuntime,
            mip_filter: TextureMipFilter::Kaiser,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/runtime-mips.png",
            "rgba8unorm_srgb",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("mip_filter=box")
        }));
    }

    #[test]
    fn runtime_mip_policy_rejects_non_rgba8_storage_formats() {
        let metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Hdr,
            color_space: RenderImageColorSpace::Linear,
            mip_policy: TextureMipPolicy::GenerateRuntime,
            mip_filter: TextureMipFilter::Box,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/runtime-hdr-mips.exr",
            "rgba16float",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic
                    .message
                    .contains("runtime mip generation supports only rgba8unorm storage")
        }));
    }

    #[test]
    fn runtime_mip_policy_accepts_rgba8_storage_formats() {
        let metadata = TextureMetadata {
            mip_policy: TextureMipPolicy::GenerateRuntime,
            mip_filter: TextureMipFilter::Box,
            ..TextureMetadata::default()
        };
        let diagnostics = validate_texture_metadata(
            "textures/runtime-mips.png",
            "rgba8unorm_srgb",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(!diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.severity == TextureMetadataDiagnosticSeverity::Error }));
    }

    #[test]
    fn invalid_anisotropy_is_an_error() {
        let metadata = TextureMetadata {
            max_anisotropy: 3,
            ..TextureMetadata::default()
        };

        let diagnostics = validate_texture_metadata(
            "textures/invalid-anisotropy.png",
            "rgba8unorm_srgb",
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("max_anisotropy")
        }));
    }
}
