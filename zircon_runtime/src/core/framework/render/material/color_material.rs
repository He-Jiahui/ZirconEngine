use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

use super::{RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialFallbackPolicy};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
