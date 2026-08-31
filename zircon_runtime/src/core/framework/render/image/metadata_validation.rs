mod format;

#[cfg(test)]
mod tests;

use self::format::{
    compression_name, format_has_srgb_variant, format_is_float_family,
    format_supports_runtime_mip_generation,
};
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
