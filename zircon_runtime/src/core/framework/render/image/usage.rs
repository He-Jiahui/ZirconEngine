use serde::{Deserialize, Serialize};

/// 资产对 GPU 图像的预期操作，由纹理准备阶段映射到后端用途并按格式能力筛选。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageUsage {
    Sampled,
    Storage,
    RenderTarget,
    CopySrc,
    CopyDst,
}
