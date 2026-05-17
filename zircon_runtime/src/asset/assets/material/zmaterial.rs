use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

use super::MaterialTextureSlotValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZMaterialDocument {
    #[serde(default = "default_zmaterial_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub shader: AssetReference,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(alias = "property_overrides")]
    pub overrides: BTreeMap<String, toml::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub textures: BTreeMap<String, MaterialTextureSlotValue>,
    #[serde(default, skip_serializing_if = "toml::Table::is_empty")]
    pub editor: toml::Table,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_diagnostics: Vec<String>,
}

impl ZMaterialDocument {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

fn default_zmaterial_version() -> u32 {
    1
}
