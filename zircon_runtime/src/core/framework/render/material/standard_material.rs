use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

use super::{RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialFallbackPolicy};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandardMaterialDescriptor {
    pub name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub base_color: [f32; 4],
    pub base_color_texture: Option<AssetReference>,
    pub normal_texture: Option<AssetReference>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<AssetReference>,
    pub occlusion_texture: Option<AssetReference>,
    pub emissive: [f32; 3],
    pub emissive_texture: Option<AssetReference>,
    pub alpha_mode: RenderMaterialAlphaMode,
    pub unlit: bool,
    pub double_sided: bool,
    pub fallback_policy: RenderMaterialFallbackPolicy,
}
