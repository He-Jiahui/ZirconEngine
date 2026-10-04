//! First-party 2D render components stored in a canonical scene record.

use crate::asset::AssetReference;
use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteImageMode,
    RenderSpriteRect,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneSprite2dAsset {
    pub image: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atlas_region: Option<RenderSpriteAtlasRegion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<RenderSpriteRect>,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub anchor: RenderSpriteAnchor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_size: Option<[f32; 2]>,
    #[serde(default)]
    pub image_mode: RenderSpriteImageMode,
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub z_order: i32,
    #[serde(default = "default_sprite_alpha_mode")]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMesh2dAsset {
    pub model: AssetReference,
    pub material: AssetReference,
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub z_order: i32,
    #[serde(default)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

fn default_color() -> [f32; 4] {
    [1.0; 4]
}

fn default_sprite_alpha_mode() -> RenderMaterialAlphaMode {
    RenderMaterialAlphaMode::Blend
}
