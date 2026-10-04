use serde::{Deserialize, Serialize};

use super::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage, RenderSamplerDescriptor, TextureMetadata,
};

/// 图像描述是可序列化资源 DTO；尺寸、维度、格式、用途和采样器共同构成创建契约，未知字段拒绝以避免静默改变资源含义。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderImageDescriptor {
    pub width: u32,
    pub height: u32,
    /// Bevy-style extent depth, or array-layer count for 1D/2D array textures.
    #[serde(default = "default_depth_or_array_layers")]
    pub depth_or_array_layers: u32,
    #[serde(default)]
    pub dimension: RenderImageDimension,
    pub format: String,
    pub color_space: RenderImageColorSpace,
    #[serde(default)]
    pub metadata: TextureMetadata,
    pub sampler: RenderSamplerDescriptor,
    pub usage: Vec<RenderImageUsage>,
    #[serde(default)]
    pub asset_usage: Vec<RenderImageAssetUsage>,
    pub mip_count: u32,
    pub fallback: RenderImageFallbackKind,
}

fn default_depth_or_array_layers() -> u32 {
    1
}
