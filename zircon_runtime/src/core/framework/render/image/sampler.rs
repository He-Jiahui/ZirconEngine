use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderSamplerAddressMode {
    #[default]
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderSamplerFilter {
    Nearest,
    #[default]
    Linear,
}

/// 纹理导入配置与 GPU 采样器缓存之间的稳定描述；纹理元数据校验会限制各向异性组合。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSamplerDescriptor {
    pub address_mode_u: RenderSamplerAddressMode,
    pub address_mode_v: RenderSamplerAddressMode,
    pub address_mode_w: RenderSamplerAddressMode,
    pub mag_filter: RenderSamplerFilter,
    pub min_filter: RenderSamplerFilter,
    pub mipmap_filter: RenderSamplerFilter,
}

impl Default for RenderSamplerDescriptor {
    fn default() -> Self {
        Self {
            address_mode_u: RenderSamplerAddressMode::ClampToEdge,
            address_mode_v: RenderSamplerAddressMode::ClampToEdge,
            address_mode_w: RenderSamplerAddressMode::ClampToEdge,
            mag_filter: RenderSamplerFilter::Linear,
            min_filter: RenderSamplerFilter::Linear,
            mipmap_filter: RenderSamplerFilter::Linear,
        }
    }
}
