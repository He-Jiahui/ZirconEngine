use serde::{Deserialize, Serialize};

use crate::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: Option<String>,
    pub shader: AssetReference,
    pub base_color: [f32; 4],
    pub base_color_texture: Option<AssetReference>,
    pub normal_texture: Option<AssetReference>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<AssetReference>,
    pub occlusion_texture: Option<AssetReference>,
    pub emissive: [f32; 3],
    pub emissive_texture: Option<AssetReference>,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

impl MaterialAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
