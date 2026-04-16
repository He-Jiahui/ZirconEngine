use serde::{Deserialize, Serialize};
use zircon_math::Real;

use crate::AssetReference;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformAsset {
    pub translation: [Real; 3],
    pub rotation: [Real; 4],
    pub scale: [Real; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneCameraAsset {
    pub fov_y_radians: Real,
    pub z_near: Real,
    pub z_far: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    pub material: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDirectionalLightAsset {
    pub direction: [Real; 3],
    pub color: [Real; 3],
    pub intensity: Real,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SceneMobilityAsset {
    #[default]
    Dynamic,
    Static,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityAsset {
    pub entity: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub transform: TransformAsset,
    #[serde(default = "default_scene_active")]
    pub active: bool,
    #[serde(default = "default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: SceneMobilityAsset,
    pub camera: Option<SceneCameraAsset>,
    pub mesh: Option<SceneMeshInstanceAsset>,
    pub directional_light: Option<SceneDirectionalLightAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAsset {
    pub entities: Vec<SceneEntityAsset>,
}

impl SceneAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

const fn default_scene_active() -> bool {
    true
}

const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}
