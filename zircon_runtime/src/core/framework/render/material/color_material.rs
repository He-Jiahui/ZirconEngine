use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

use super::{RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialFallbackPolicy};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// MaterialAsset 导出的简化颜色材质视图；这里只携带依赖与表面参数，资源解析仍由渲染准备阶段负责。
pub struct ColorMaterialDescriptor {
    pub name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub color: [f32; 4],
    pub texture: Option<AssetReference>,
    pub alpha_mode: RenderMaterialAlphaMode,
    pub unlit: bool,
    pub double_sided: bool,
    pub fallback_policy: RenderMaterialFallbackPolicy,
}
