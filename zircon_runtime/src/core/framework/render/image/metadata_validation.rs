use super::{
    RenderImageColorSpace, RenderSamplerDescriptor, RenderSamplerFilter, TextureCompressionTarget,
    TextureMetadata, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
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
    if metadata.color_space == RenderImageColorSpace::Srgb && !format_has_srgb_variant(format) {
        error(
            &mut diagnostics,
            format!("format '{format}' has no srgb variant: '{uri}'"),
        );
    }
    diagnostics
}

fn format_has_srgb_variant(format: &str) -> bool {
    let format = format.trim().to_ascii_lowercase();
    format.contains("srgb")
        || matches!(
            format.as_str(),
            "rgba8unorm" | "bgra8unorm" | "bc1" | "bc2" | "bc3" | "bc7"
        )
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

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        );
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
}
