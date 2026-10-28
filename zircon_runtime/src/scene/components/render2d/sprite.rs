use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteImageMode,
    RenderSpriteRect,
};
use crate::core::math::{Vec2, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, TextureMarker};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::Sprite2dComponent"
)]
/// 场景精灵的持久化配置；图集裁剪和材质选择由渲染提取解释，创建节点时只需填入资源句柄与局部绘制参数。
pub struct Sprite2dComponent {
    #[zr_reflect(
        value_type_path = "Resource",
        read = "super::sprite_reflection::read_image",
        readonly
    )]
    pub image: ResourceHandle<TextureMarker>,
    #[zr_reflect(
        value_type_path = "Resource",
        read = "super::sprite_reflection::read_material",
        readonly
    )]
    pub material: Option<ResourceHandle<MaterialMarker>>,
    #[zr_reflect(skip)]
    pub atlas_region: Option<RenderSpriteAtlasRegion>,
    #[zr_reflect(skip)]
    pub rect: Option<RenderSpriteRect>,
    #[zr_reflect(readonly)]
    pub flip_x: bool,
    #[zr_reflect(readonly)]
    pub flip_y: bool,
    #[zr_reflect(skip)]
    pub anchor: RenderSpriteAnchor,
    #[zr_reflect(
        value_type_path = "Vec2",
        read = "super::sprite_reflection::read_custom_size",
        readonly
    )]
    pub custom_size: Option<Vec2>,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub image_mode: RenderSpriteImageMode,
    #[zr_reflect(readonly)]
    pub color: Vec4,
    #[zr_reflect(readonly)]
    pub z_order: i32,
    #[serde(default)]
    #[zr_reflect(skip)]
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
            image_mode: RenderSpriteImageMode::Stretch,
            color: Vec4::ONE,
            z_order: 0,
            material_alpha_mode: RenderMaterialAlphaMode::Blend,
        }
    }
}
