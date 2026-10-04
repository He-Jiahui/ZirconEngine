use serde::{Deserialize, Serialize};

use crate::core::math::Vec2;
use crate::core::resource::ResourceId as AssetId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CookieWrapMode {
    #[default]
    Clamp,
    Repeat,
}

/// 描述不同灯型如何投影采样 cookie；图形层依据灯光类型打包贴图索引和投影参数。
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

/// 场景提取传给光源打包器的 cookie 引用；light_id 必须对应同一帧的灯光快照，纹理驻留由图形层决定。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightCookieData {
    pub light_id: u64,
    pub texture: AssetId,
    pub projection: CookieProjection,
}
