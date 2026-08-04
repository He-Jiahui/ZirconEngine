use serde::{Deserialize, Serialize};

use super::RenderImageColorSpace;

/// Shared SVT source-page contract consumed by importers and runtime page tables.
pub const TEXTURE_SVT_DEFAULT_PAGE_SIZE: u32 = 128;
/// Shared texel border around each virtual-texture page payload.
pub const TEXTURE_SVT_DEFAULT_BORDER_SIZE: u32 = 4;
pub const TEXTURE_DEFAULT_MAX_ANISOTROPY: u8 = 1;
/// Small images stay fully resident because streaming them costs more than it saves.
pub const TEXTURE_STREAMING_MIN_DIMENSION: u32 = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureMipPolicy {
    #[default]
    FromSource,
    GenerateOffline,
    GenerateRuntime,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureMipFilter {
    #[default]
    Kaiser,
    Box,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureNormalConvention {
    #[default]
    None,
    TangentSpaceDx,
    TangentSpaceGl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureUsageHint {
    #[default]
    Albedo,
    Normal,
    Mask,
    Data,
    Hdr,
    Ui,
}

pub fn default_color_space_for_texture_usage(
    usage_hint: TextureUsageHint,
) -> RenderImageColorSpace {
    match usage_hint {
        TextureUsageHint::Albedo | TextureUsageHint::Ui => RenderImageColorSpace::Srgb,
        TextureUsageHint::Normal
        | TextureUsageHint::Mask
        | TextureUsageHint::Data
        | TextureUsageHint::Hdr => RenderImageColorSpace::Linear,
    }
}

pub fn default_mip_filter_for_texture_usage(usage_hint: TextureUsageHint) -> TextureMipFilter {
    match usage_hint {
        TextureUsageHint::Normal | TextureUsageHint::Ui => TextureMipFilter::Box,
        TextureUsageHint::Albedo
        | TextureUsageHint::Mask
        | TextureUsageHint::Data
        | TextureUsageHint::Hdr => TextureMipFilter::Kaiser,
    }
}

pub fn default_compression_for_texture_usage(
    usage_hint: TextureUsageHint,
) -> TextureCompressionTarget {
    match usage_hint {
        TextureUsageHint::Albedo | TextureUsageHint::Data => TextureCompressionTarget::Bc7,
        TextureUsageHint::Normal => TextureCompressionTarget::Bc5,
        TextureUsageHint::Mask => TextureCompressionTarget::Bc4,
        TextureUsageHint::Hdr => TextureCompressionTarget::Bc6h,
        TextureUsageHint::Ui => TextureCompressionTarget::Uncompressed,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureCompressionTarget {
    #[default]
    Auto,
    Uncompressed,
    Bc1,
    Bc4,
    Bc5,
    Bc6h,
    Bc7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvtSettings {
    pub page_size: u32,
    pub border_size: u32,
    pub mip_tail_first_level: u32,
}

impl Default for SvtSettings {
    fn default() -> Self {
        Self {
            page_size: TEXTURE_SVT_DEFAULT_PAGE_SIZE,
            border_size: TEXTURE_SVT_DEFAULT_BORDER_SIZE,
            mip_tail_first_level: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub color_space: RenderImageColorSpace,
    pub usage_hint: TextureUsageHint,
    pub mip_policy: TextureMipPolicy,
    #[serde(default)]
    pub mip_filter: TextureMipFilter,
    pub normal_convention: TextureNormalConvention,
    pub compression: TextureCompressionTarget,
    pub mip_bias: f32,
    pub max_anisotropy: u8,
    /// Enables ordinary distance-driven mip streaming when this asset is eligible.
    #[serde(default = "default_texture_streaming_enabled")]
    pub streaming_enabled: bool,
    pub svt: Option<SvtSettings>,
}

impl PartialEq for TextureMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.color_space == other.color_space
            && self.usage_hint == other.usage_hint
            && self.mip_policy == other.mip_policy
            && self.mip_filter == other.mip_filter
            && self.normal_convention == other.normal_convention
            && self.compression == other.compression
            && self.mip_bias.to_bits() == other.mip_bias.to_bits()
            && self.max_anisotropy == other.max_anisotropy
            && self.streaming_enabled == other.streaming_enabled
            && self.svt == other.svt
    }
}

// Texture descriptors are content-addressed; bitwise comparison keeps NaN payloads reflexive.
impl Eq for TextureMetadata {}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self {
            color_space: default_color_space_for_texture_usage(TextureUsageHint::Albedo),
            usage_hint: TextureUsageHint::Albedo,
            mip_policy: TextureMipPolicy::FromSource,
            mip_filter: default_mip_filter_for_texture_usage(TextureUsageHint::Albedo),
            normal_convention: TextureNormalConvention::None,
            compression: TextureCompressionTarget::Auto,
            mip_bias: 0.0,
            max_anisotropy: TEXTURE_DEFAULT_MAX_ANISOTROPY,
            streaming_enabled: default_texture_streaming_enabled(),
            svt: None,
        }
    }
}

impl TextureMetadata {
    /// SVT and generated/UI/small textures always use the fully-resident fallback path.
    pub const fn allows_mip_streaming(&self, width: u32, height: u32, mip_count: u32) -> bool {
        self.streaming_enabled
            && !matches!(self.usage_hint, TextureUsageHint::Ui)
            && matches!(
                self.mip_policy,
                TextureMipPolicy::FromSource | TextureMipPolicy::GenerateOffline
            )
            && width >= TEXTURE_STREAMING_MIN_DIMENSION
            && height >= TEXTURE_STREAMING_MIN_DIMENSION
            && mip_count > 1
            && self.svt.is_none()
    }
}

const fn default_texture_streaming_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_mip_streaming_metadata_defaults_enabled_and_exempts_ineligible_assets() {
        let metadata = TextureMetadata::default();
        assert!(metadata.streaming_enabled);
        assert!(metadata.allows_mip_streaming(1024, 1024, 8));

        let ui = TextureMetadata {
            usage_hint: TextureUsageHint::Ui,
            ..TextureMetadata::default()
        };
        assert!(!ui.allows_mip_streaming(1024, 1024, 8));

        assert!(!metadata.allows_mip_streaming(128, 128, 8));
        assert!(!metadata.allows_mip_streaming(1024, 1024, 1));

        let disabled = TextureMetadata {
            streaming_enabled: false,
            ..TextureMetadata::default()
        };
        assert!(!disabled.allows_mip_streaming(1024, 1024, 8));

        let generated = TextureMetadata {
            mip_policy: TextureMipPolicy::GenerateRuntime,
            ..TextureMetadata::default()
        };
        assert!(!generated.allows_mip_streaming(1024, 1024, 8));

        let svt = TextureMetadata {
            svt: Some(SvtSettings::default()),
            ..TextureMetadata::default()
        };
        assert!(!svt.allows_mip_streaming(4096, 4096, 12));
    }
}
