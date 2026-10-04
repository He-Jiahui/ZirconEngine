use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderMaterialAlphaMode, RendererCommon};
use crate::core::framework::scene::EntityId;
use crate::core::math::{Transform, Vec2, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, TextureMarker};

use super::{RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteImageMode, RenderSpriteRect};

/// 场景组件抽取后的帧级绘制输入；资源句柄标识纹理/材质，队列另行决定绘制阶段和顺序。
/// 顶点构建还会按相机层、颜色和尺寸过滤，不能把快照存在等同于最终可见。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteSnapshot {
    pub entity: EntityId,
    pub transform: Transform,
    pub image: ResourceHandle<TextureMarker>,
    pub material: Option<ResourceHandle<MaterialMarker>>,
    pub atlas_region: Option<RenderSpriteAtlasRegion>,
    pub rect: Option<RenderSpriteRect>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: RenderSpriteAnchor,
    pub custom_size: Option<Vec2>,
    #[serde(default)]
    pub image_mode: RenderSpriteImageMode,
    pub color: Vec4,
    pub z_order: i32,
    #[serde(default)]
    pub common: RendererCommon,
    pub material_alpha_mode: RenderMaterialAlphaMode,
}
