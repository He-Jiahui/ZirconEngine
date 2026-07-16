use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{
    default_ambient_light_intensity, default_light_color, default_rect_light_intensity,
    default_rect_light_range, default_rect_light_size, default_true, default_vec3_up,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAmbientLightAsset {
    #[serde(default = "default_light_color")]
    pub color: [Real; 3],
    #[serde(default = "default_ambient_light_intensity")]
    pub intensity: Real,
    #[serde(default = "default_true")]
    pub affects_lightmapped_meshes: bool,
}

impl Default for SceneAmbientLightAsset {
    fn default() -> Self {
        Self {
            color: default_light_color(),
            intensity: default_ambient_light_intensity(),
            affects_lightmapped_meshes: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDirectionalLightAsset {
    pub direction: [Real; 3],
    pub color: [Real; 3],
    pub intensity: Real,
    #[serde(default)]
    pub volumetric: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenePointLightAsset {
    pub color: [Real; 3],
    pub intensity: Real,
    pub range: Real,
    #[serde(default)]
    pub volumetric: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneSpotLightAsset {
    #[serde(default = "default_vec3_up")]
    pub direction: [Real; 3],
    #[serde(default = "default_light_color")]
    pub color: [Real; 3],
    #[serde(default = "default_rect_light_intensity")]
    pub intensity: Real,
    #[serde(default = "default_rect_light_range")]
    pub range: Real,
    #[serde(default)]
    pub inner_angle_radians: Real,
    #[serde(default)]
    pub outer_angle_radians: Real,
    #[serde(default)]
    pub volumetric: bool,
}

impl Default for SceneSpotLightAsset {
    fn default() -> Self {
        Self {
            direction: default_vec3_up(),
            color: default_light_color(),
            intensity: default_rect_light_intensity(),
            range: default_rect_light_range(),
            inner_angle_radians: 0.0,
            outer_angle_radians: 0.0,
            volumetric: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneRectLightAsset {
    #[serde(default = "default_light_color")]
    pub color: [Real; 3],
    #[serde(default = "default_rect_light_intensity")]
    pub intensity: Real,
    #[serde(default = "default_rect_light_range")]
    pub range: Real,
    #[serde(default = "default_rect_light_size")]
    pub size: [Real; 2],
    #[serde(default)]
    pub volumetric: bool,
}

impl Default for SceneRectLightAsset {
    fn default() -> Self {
        Self {
            color: default_light_color(),
            intensity: default_rect_light_intensity(),
            range: default_rect_light_range(),
            size: default_rect_light_size(),
            volumetric: false,
        }
    }
}
