use serde::{Deserialize, Serialize};

use crate::core::math::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteBounds {
    pub center: Vec3,
    pub half_size: Vec2,
}

impl RenderSpriteBounds {
    pub const fn new(center: Vec3, half_size: Vec2) -> Self {
        Self { center, half_size }
    }
}
