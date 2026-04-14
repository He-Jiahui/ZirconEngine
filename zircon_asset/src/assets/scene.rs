use serde::{Deserialize, Serialize};

use crate::AssetReference;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformAsset {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneCameraAsset {
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    pub material: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDirectionalLightAsset {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityAsset {
    pub entity: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub transform: TransformAsset,
    pub active: bool,
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
