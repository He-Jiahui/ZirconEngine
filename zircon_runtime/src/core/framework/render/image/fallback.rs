use serde::{Deserialize, Serialize};

/// 资源缺失或尚未准备完成时使用的语义占位；创建 GPU 回退纹理时须保持相应的颜色解释。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageFallbackKind {
    #[default]
    MissingImage,
    OpaqueWhite,
    TransparentBlack,
    NormalMap,
}
