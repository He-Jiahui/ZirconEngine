use serde::{Deserialize, Serialize};

use super::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage, RenderSamplerDescriptor,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub sampler: RenderSamplerDescriptor,
    pub usage: Vec<RenderImageUsage>,
    #[serde(default)]
    pub asset_usage: Vec<RenderImageAssetUsage>,
    pub mip_count: u32,
    pub array_layer_count: u32,
    pub fallback: RenderImageFallbackKind,
}

fn default_depth_or_array_layers() -> u32 {
    1
}
