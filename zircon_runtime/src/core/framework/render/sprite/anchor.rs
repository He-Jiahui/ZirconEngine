use serde::{Deserialize, Serialize};

use crate::core::math::Vec2;

/// 精灵局部四边形的归一化锚点；顶点构建以左下为原点，Y 向上，场景变换随后生效。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteAnchor {
    pub normalized: Vec2,
}

impl RenderSpriteAnchor {
    pub const CENTER: Self = Self {
        normalized: Vec2::new(0.5, 0.5),
    };
    pub const TOP_LEFT: Self = Self {
        normalized: Vec2::new(0.0, 1.0),
    };
    pub const TOP_RIGHT: Self = Self {
        normalized: Vec2::new(1.0, 1.0),
    };
    pub const BOTTOM_LEFT: Self = Self {
        normalized: Vec2::new(0.0, 0.0),
    };
    pub const BOTTOM_RIGHT: Self = Self {
        normalized: Vec2::new(1.0, 0.0),
    };

    pub const fn new(normalized: Vec2) -> Self {
        Self { normalized }
    }
}

impl Default for RenderSpriteAnchor {
    fn default() -> Self {
        Self::CENTER
    }
}
