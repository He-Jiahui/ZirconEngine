use serde::{Deserialize, Serialize};

use super::{
    RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage, RenderSamplerDescriptor,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderImageDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub color_space: RenderImageColorSpace,
    pub sampler: RenderSamplerDescriptor,
    pub usage: Vec<RenderImageUsage>,
    pub mip_count: u32,
    pub array_layer_count: u32,
    pub fallback: RenderImageFallbackKind,
}
