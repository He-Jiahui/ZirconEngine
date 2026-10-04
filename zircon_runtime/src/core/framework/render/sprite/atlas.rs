use serde::{Deserialize, Serialize};

use crate::core::math::Vec2;

/// 图集内的归一化 UV 子区域；顶点构建把源矩形映射到这里，翻转在映射之后处理。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteAtlasRegion {
    pub min: Vec2,
    pub max: Vec2,
}

impl RenderSpriteAtlasRegion {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}
