use serde::{Deserialize, Serialize};

use crate::core::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl RenderSpriteRect {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}
