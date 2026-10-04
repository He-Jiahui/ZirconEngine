use serde::{Deserialize, Serialize};

/// 导入与 GPU 视图共享的颜色解释契约；颜色贴图使用 sRGB，法线和数据贴图保持线性。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageColorSpace {
    #[default]
    Srgb,
    Linear,
    Hdr,
}
