use serde::{Deserialize, Serialize};

use super::RenderImageColorSpace;

/// Shared SVT source-page contract consumed by importers and runtime page tables.
pub const TEXTURE_SVT_DEFAULT_PAGE_SIZE: u32 = 128;
/// Shared texel border around each virtual-texture page payload.
pub const TEXTURE_SVT_DEFAULT_BORDER_SIZE: u32 = 4;
pub const TEXTURE_DEFAULT_MAX_ANISOTROPY: u8 = 1;

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
    pub normal_convention: TextureNormalConvention,
    pub compression: TextureCompressionTarget,
    pub mip_bias: f32,
    pub max_anisotropy: u8,
    pub svt: Option<SvtSettings>,
}

impl PartialEq for TextureMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.color_space == other.color_space
            && self.usage_hint == other.usage_hint
            && self.mip_policy == other.mip_policy
            && self.normal_convention == other.normal_convention
            && self.compression == other.compression
            && self.mip_bias.to_bits() == other.mip_bias.to_bits()
            && self.max_anisotropy == other.max_anisotropy
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
            normal_convention: TextureNormalConvention::None,
            compression: TextureCompressionTarget::Auto,
            mip_bias: 0.0,
            max_anisotropy: TEXTURE_DEFAULT_MAX_ANISOTROPY,
            svt: None,
        }
    }
}
