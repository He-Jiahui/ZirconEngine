use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteRect,
};
use crate::core::math::{Vec2, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, TextureMarker};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprite2dComponent {
    pub image: ResourceHandle<TextureMarker>,
    pub material: Option<ResourceHandle<MaterialMarker>>,
    pub atlas_region: Option<RenderSpriteAtlasRegion>,
    pub rect: Option<RenderSpriteRect>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: RenderSpriteAnchor,
    pub custom_size: Option<Vec2>,
    pub color: Vec4,
    pub z_order: i32,
    #[serde(default)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

impl Default for Sprite2dComponent {
    fn default() -> Self {
        Self {
            image: ResourceHandle::new(ResourceId::from_stable_label("builtin://missing-texture")),
            material: None,
            atlas_region: None,
            rect: None,
            flip_x: false,
            flip_y: false,
            anchor: RenderSpriteAnchor::CENTER,
            custom_size: None,
            color: Vec4::ONE,
            z_order: 0,
            material_alpha_mode: RenderMaterialAlphaMode::Blend,
        }
    }
}
