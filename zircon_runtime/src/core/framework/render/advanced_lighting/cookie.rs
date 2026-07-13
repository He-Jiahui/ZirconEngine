use serde::{Deserialize, Serialize};

use crate::core::math::Vec2;
use crate::core::resource::ResourceId as AssetId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CookieWrapMode {
    #[default]
    Clamp,
    Repeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CookieProjection {
    Directional {
        offset: Vec2,
        scale: Vec2,
        wrap: CookieWrapMode,
    },
    Spot,
    PointOctahedral,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightCookieData {
    pub light_id: u64,
    pub texture: AssetId,
    pub projection: CookieProjection,
}
