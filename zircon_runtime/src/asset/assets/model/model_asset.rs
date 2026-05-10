use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::RenderMeshDescriptor;

use super::ModelPrimitiveAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAsset {
    pub uri: AssetUri,
    pub primitives: Vec<ModelPrimitiveAsset>,
}

impl ModelAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn render_mesh_descriptors(&self) -> Vec<RenderMeshDescriptor> {
        self.primitives
            .iter()
            .map(ModelPrimitiveAsset::render_mesh_descriptor)
            .collect()
    }
}
